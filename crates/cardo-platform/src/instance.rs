use anyhow::{Context, Result, bail};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::windows::io::{AsRawHandle, FromRawHandle},
};
use windows_sys::Win32::{
    Foundation::{
        ERROR_ACCESS_DENIED, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE,
    },
    Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_INBOUND},
    System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_REJECT_REMOTE_CLIENTS,
        WaitNamedPipeW,
    },
};

pub fn instance(application_id: &str, args: &[String]) -> Result<Option<Instance>> {
    anyhow::ensure!(
        !application_id.is_empty()
            && application_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.'),
        "Invalid instance identity"
    );
    let user = std::env::var("USERNAME").context("Cannot determine the Windows user")?;
    let name = format!(r"\\.\pipe\{application_id}-{user}");
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    // The first pipe instance owns the application session; subsequent launches forward arguments.
    let handle = unsafe {
        CreateNamedPipeW(
            wide.as_ptr(),
            PIPE_ACCESS_INBOUND | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_REJECT_REMOTE_CLIENTS,
            1,
            0,
            65536,
            5000,
            std::ptr::null(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        let error = std::io::Error::last_os_error();
        if !matches!(error.raw_os_error(),Some(code)if code==ERROR_ACCESS_DENIED as i32||code==ERROR_PIPE_BUSY as i32)
        {
            return Err(error.into());
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut pipe = loop {
            match OpenOptions::new().write(true).open(&name) {
                Ok(pipe) => break pipe,
                Err(error)
                    if error.raw_os_error() == Some(ERROR_PIPE_BUSY as i32)
                        && std::time::Instant::now() < deadline =>
                {
                    let remaining = deadline
                        .saturating_duration_since(std::time::Instant::now())
                        .as_millis()
                        .max(1) as u32;
                    if unsafe { WaitNamedPipeW(wide.as_ptr(), remaining) } == 0 {
                        return Err(std::io::Error::last_os_error().into());
                    }
                }
                Err(error) => return Err(error.into()),
            }
        };
        let payload = serde_json::to_vec(args)?;
        if payload.len() > 65536 {
            bail!("Launch payload exceeds 65536 bytes");
        }
        pipe.write_all(&(payload.len() as u32).to_le_bytes())?;
        pipe.write_all(&payload)?;
        return Ok(None);
    }
    // Transfer the owned pipe handle to File; its drop closes the handle once.
    let mut pipe = unsafe { File::from_raw_handle(handle) };
    let (tx, rx) = async_channel::unbounded();
    let events = tx;
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stopping = stop.clone();
    let worker = std::thread::spawn(move || {
        while !stopping.load(std::sync::atomic::Ordering::Relaxed) {
            let handle = pipe.as_raw_handle();
            if unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) } == 0
                && std::io::Error::last_os_error().raw_os_error()
                    != Some(ERROR_PIPE_CONNECTED as i32)
            {
                let _ = events.send_blocking(LaunchEvent::Error(
                    std::io::Error::last_os_error().to_string(),
                ));
                break;
            }
            if stopping.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            let result = (|| -> Result<Vec<String>> {
                let mut length = [0; 4];
                pipe.read_exact(&mut length)?;
                let length = u32::from_le_bytes(length) as usize;
                if length > 65536 {
                    bail!("Launch payload exceeds 65536 bytes");
                }
                let mut bytes = vec![0; length];
                pipe.read_exact(&mut bytes)?;
                Ok(serde_json::from_slice(&bytes)?)
            })();
            unsafe { DisconnectNamedPipe(handle) };
            let command = match result {
                Ok(args) => LaunchEvent::Launch(args),
                Err(error) => LaunchEvent::Error(error.to_string()),
            };
            if events.send_blocking(command).is_err() {
                break;
            }
        }
    });
    Ok(Some(Instance {
        receiver: rx,
        stop,
        worker: Some(worker),
    }))
}

pub enum LaunchEvent {
    Launch(Vec<String>),
    Error(String),
}
pub struct Instance {
    pub receiver: async_channel::Receiver<LaunchEvent>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Drop for Instance {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        self.receiver.close();
        if let Some(worker) = self.worker.take() {
            // Cancel both connection waits and reads from clients that never finish a payload.
            // Repeat until exit to cover a cancellation racing with the next synchronous call.
            while !worker.is_finished() {
                unsafe {
                    windows_sys::Win32::System::IO::CancelSynchronousIo(worker.as_raw_handle());
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            let _ = worker.join();
        }
    }
}
