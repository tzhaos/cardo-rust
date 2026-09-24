use crate::UpdateService;
use anyhow::{Context, Result, bail, ensure};
use cardo_runtime::messages::{tf, tr};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::windows::{
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
        QueryFullProcessImageNameW, WaitForSingleObject,
    },
};

pub struct Target {
    directory: PathBuf,
    registration: Option<serde_json::Value>,
}

impl Target {
    pub fn installed(&self) -> bool {
        self.registration.is_some()
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Backup {
    name: String,
    existed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Plan {
    target: PathBuf,
    job: PathBuf,
    stage: PathBuf,
    package: String,
    digest: String,
    version: String,
    parent_pid: u32,
    helper_pid: u32,
    installer_pid: u32,
    registration: Option<serde_json::Value>,
    backup: Vec<Backup>,
    applying: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Outcome {
    job: PathBuf,
    helper_pid: u32,
    error: Option<String>,
}

pub struct Prepared {
    job: tempfile::TempDir,
    stage: tempfile::TempDir,
    plan: Plan,
    service: UpdateService,
}

impl UpdateService {
    pub fn target(&self) -> Result<Target> {
        let executable = std::env::current_exe()?;
        let directory = executable
            .parent()
            .context(tr("executable-directory-missing"))?
            .to_owned();
        let registration = self.installer.registration(&directory)?;
        Ok(Target {
            directory,
            registration,
        })
    }
    pub fn job_directory(&self) -> Result<tempfile::TempDir> {
        let root = self.config.jobs.clone();
        fs::create_dir_all(&root).with_context(|| format!("Cannot create {}", root.display()))?;
        Ok(tempfile::Builder::new().prefix("job-").tempdir_in(root)?)
    }

    pub fn sha256(path: &Path) -> Result<String> {
        let mut file =
            File::open(path).with_context(|| format!("Cannot read {}", path.display()))?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        Ok(format!("{:x}", hash.finalize()))
    }

    pub fn prepare(
        &self,
        target: Target,
        job: tempfile::TempDir,
        package: &str,
        digest: String,
        version: String,
        cancel: &cardo_runtime::task::CancellationToken,
    ) -> Result<Prepared> {
        ensure!(
            Self::sha256(&job.path().join(package))? == digest,
            tr("update-hash-mismatch")
        );
        let stage = tempfile::Builder::new()
            .prefix(&self.config.stage_prefix)
            .tempdir_in(&target.directory)
            .with_context(|| format!("Cannot stage update in {}", target.directory.display()))?;
        if !target.installed() {
            self.unpack(&job.path().join(package), &stage.path().join("new"), cancel)?;
        }
        let plan = Plan {
            target: target.directory,
            job: job.path().to_owned(),
            stage: stage.path().to_owned(),
            package: package.into(),
            digest,
            version,
            parent_pid: std::process::id(),
            helper_pid: 0,
            installer_pid: 0,
            registration: target.registration,
            backup: Vec::new(),
            applying: false,
        };
        Ok(Prepared {
            job,
            stage,
            plan,
            service: self.clone(),
        })
    }

    fn unpack(
        &self,
        package: &Path,
        destination: &Path,
        cancel: &cardo_runtime::task::CancellationToken,
    ) -> Result<()> {
        let mut zip = zip::ZipArchive::new(File::open(package)?)?;
        let mut seen = std::collections::BTreeSet::new();
        let mut total = 0u64;
        for index in 0..zip.len() {
            ensure!(!cancel.is_cancelled(), tr("task-cancelled"));
            let mut entry = zip.by_index(index)?;
            let enclosed = entry
                .enclosed_name()
                .context(tr("update-package-invalid"))?;
            ensure!(
                !entry
                    .unix_mode()
                    .is_some_and(|mode| mode & 0o170000 == 0o120000),
                tr("update-package-invalid")
            );
            let relative = enclosed
                .strip_prefix(&self.config.archive_root)
                .context(tr("update-package-invalid"))?;
            if entry.is_dir() {
                continue;
            }
            let name = relative.to_string_lossy().replace('\\', "/");
            ensure!(
                self.config.files.contains(&name) && seen.insert(name),
                tr("update-package-invalid")
            );
            total = total
                .checked_add(entry.size())
                .context(tr("update-package-invalid"))?;
            ensure!(
                total <= self.config.max_package_bytes,
                tr("update-package-invalid")
            );
            let output = destination.join(relative);
            fs::create_dir_all(output.parent().unwrap())?;
            let mut file = File::create(&output)?;
            let mut buffer = [0u8; 65536];
            loop {
                ensure!(!cancel.is_cancelled(), tr("task-cancelled"));
                let count = entry.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                file.write_all(&buffer[..count])?;
            }
            file.sync_all()?;
        }
        ensure!(
            seen.len() == self.config.files.len(),
            tr("update-package-invalid")
        );
        Ok(())
    }

    fn validate_plan(&self, plan: &Plan) -> Result<()> {
        let root = self.config.jobs.clone();
        ensure!(
            plan.job.parent() == Some(root.as_path())
                && plan
                    .job
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("job-")),
            tr("update-package-invalid")
        );
        ensure!(
            plan.target.is_absolute()
                && plan.stage.parent() == Some(plan.target.as_path())
                && plan
                    .stage
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with(&self.config.stage_prefix)),
            tr("update-package-invalid")
        );
        ensure!(
            plan.package == self.config.installer || plan.package == self.config.portable,
            tr("update-package-invalid")
        );
        Ok(())
    }

    fn safe_target(root: &Path, name: &str) -> Result<PathBuf> {
        let relative = Path::new(name);
        ensure!(
            relative
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_))),
            tr("update-package-invalid")
        );
        let path = root.join(relative);
        if path.exists() {
            ensure!(
                !fs::symlink_metadata(&path)?.file_type().is_symlink(),
                tr("update-package-invalid")
            );
            ensure!(
                fs::canonicalize(&path)?.starts_with(fs::canonicalize(root)?),
                tr("update-package-invalid")
            );
        } else {
            let mut ancestor = path.parent().context(tr("update-package-invalid"))?;
            while !ancestor.exists() {
                ancestor = ancestor.parent().context(tr("update-package-invalid"))?;
            }
            ensure!(
                fs::canonicalize(ancestor)?.starts_with(fs::canonicalize(root)?),
                tr("update-package-invalid")
            );
        }
        Ok(path)
    }

    fn atomic_copy(source: &Path, destination: &Path) -> Result<()> {
        let parent = destination.parent().context(tr("update-package-invalid"))?;
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        std::io::copy(&mut File::open(source)?, temporary.as_file_mut())?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(destination)
            .map_err(|error| error.error)
            .with_context(|| format!("Cannot replace {}", destination.display()))?;
        Ok(())
    }

    fn backup(&self, plan: &mut Plan) -> Result<()> {
        let mut names: Vec<String> = self.config.files.clone();
        if let Some(registration) = &plan.registration {
            names.extend(self.installer.backup_files(&plan.target, registration)?);
        }
        for name in names {
            let path = Self::safe_target(&plan.target, &name)?;
            let existed = match fs::metadata(&path) {
                Ok(metadata) => {
                    ensure!(metadata.is_file(), tr("update-package-invalid"));
                    true
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("Cannot inspect {}", path.display()));
                }
            };
            if existed {
                Self::atomic_copy(&path, &plan.stage.join("backup").join(&name))?;
            }
            plan.backup.push(Backup { name, existed });
        }
        plan.applying = true;
        self.journal.save_pending(&serde_json::to_value(&plan)?)?;
        Ok(())
    }

    fn rollback(&self, plan: &Plan) -> Result<()> {
        if !plan.applying {
            return Ok(());
        }
        for entry in &plan.backup {
            let target = Self::safe_target(&plan.target, &entry.name)?;
            if entry.existed {
                let source = plan.stage.join("backup").join(&entry.name);
                if target.is_file() && Self::sha256(&target)? == Self::sha256(&source)? {
                    continue;
                }
                Self::atomic_copy(&source, &target)?;
            } else if target.exists() {
                fs::remove_file(target)?;
            }
        }
        if let Some(registration) = &plan.registration {
            self.installer.restore(&plan.target, registration)?;
        }
        Ok(())
    }

    fn install(&self, plan: &mut Plan) -> Result<()> {
        ensure!(
            Self::sha256(&plan.job.join(&plan.package))? == plan.digest,
            tr("update-hash-mismatch")
        );
        if plan.registration.is_some() {
            ensure!(
                self.installer.registration(&plan.target)?.is_some(),
                tr("update-ownership-error")
            );
            let mut installer = self
                .installer
                .spawn(&plan.job.join(&plan.package), &plan.target)?;
            plan.installer_pid = installer.id();
            if let Err(error) = self.journal.save_pending(&serde_json::to_value(&plan)?) {
                let _ = installer.kill();
                let _ = installer.wait();
                return Err(error);
            }
            let status = installer.wait()?;
            ensure!(status.success(), tr("update-installer-failed"));
            self.installer.verify(&plan.target, &plan.version)?;
        } else {
            for name in &self.config.files {
                let target = Self::safe_target(&plan.target, name)?;
                Self::atomic_copy(&plan.stage.join("new").join(name), &target)?;
            }
        }
        Ok(())
    }

    pub fn apply(&self) -> Result<()> {
        let mut plan = self
            .journal
            .pending()?
            .map(serde_json::from_value::<Plan>)
            .transpose()?
            .context(tr("update-package-invalid"))?;
        self.validate_plan(&plan)?;
        let helper = fs::canonicalize(std::env::current_exe()?)?;
        let job = fs::canonicalize(&plan.job)?;
        ensure!(
            helper.parent() == Some(job.as_path()),
            tr("update-package-invalid")
        );
        let parent = process(plan.parent_pid, &plan.target.join(&self.config.executable))?
            .context(tr("update-helper-failed"))?;
        plan.helper_pid = std::process::id();
        self.journal.save_pending(&serde_json::to_value(&plan)?)?;
        File::create(plan.job.join("ready"))?.sync_all()?;
        match unsafe { WaitForSingleObject(parent.as_raw_handle(), 60000) } {
            WAIT_OBJECT_0 => {}
            WAIT_TIMEOUT => {
                self.journal.clear_pending()?;
                bail!(tr("maintenance-app-busy"));
            }
            _ => return Err(std::io::Error::last_os_error().into()),
        }
        let result = self.backup(&mut plan).and_then(|_| self.install(&mut plan));
        let error = if let Err(error) = result {
            tracing::error!(error = %format!("{error:#}"), "Update installation failed");
            if plan.installer_pid != 0
                && process(plan.installer_pid, &plan.job.join(&plan.package))?.is_some()
            {
                bail!(tr("update-running"));
            }
            self.rollback(&plan).with_context(|| {
                tf(
                    "update-rollback-failed",
                    &[("path", plan.stage.display().to_string().into())],
                )
            })?;
            Some(tf(
                "update-rolled-back",
                &[("error", format!("{error:#}").into())],
            ))
        } else {
            None
        };
        self.journal.finish(&serde_json::to_value(Outcome {
            job: plan.job.clone(),
            helper_pid: plan.helper_pid,
            error,
        })?)?;
        if let Err(error) = fs::remove_dir_all(&plan.stage) {
            tracing::warn!(error = %error, "Cannot clean update staging directory");
        }
        Command::new(plan.target.join(&self.config.executable))
            .current_dir(&plan.target)
            .spawn()
            .context(tr("update-restart-failed"))?;
        Ok(())
    }

    /// Called before opening a normal application window, never during rendering.
    pub fn recover(&self) -> Result<Option<String>> {
        if let Some(plan) = self
            .journal
            .pending()?
            .map(serde_json::from_value::<Plan>)
            .transpose()?
        {
            self.validate_plan(&plan)?;
            if plan.installer_pid != 0
                && process(plan.installer_pid, &plan.job.join(&plan.package))?.is_some()
            {
                bail!(tr("update-running"));
            }
            if plan.helper_pid != 0
                && process(plan.helper_pid, &plan.job.join("updater.exe"))?.is_some()
            {
                bail!(tr("update-running"));
            }
            if plan.helper_pid == 0
                && process(plan.parent_pid, &plan.target.join(&self.config.executable))?.is_some()
            {
                bail!(tr("update-running"));
            }
            self.rollback(&plan).with_context(|| {
                tf(
                    "update-rollback-failed",
                    &[("path", plan.stage.display().to_string().into())],
                )
            })?;
            self.journal.clear_pending()?;
            for directory in [&plan.stage, &plan.job] {
                if let Err(error) = fs::remove_dir_all(directory) {
                    tracing::warn!(path = %directory.display(), error = %error, "Cannot clean interrupted update directory");
                }
            }
            return Ok(Some(tr("update-recovered").into()));
        }
        if let Some(outcome) = self
            .journal
            .outcome()?
            .map(serde_json::from_value::<Outcome>)
            .transpose()?
        {
            let root = self.config.jobs.clone();
            if outcome.job.parent() == Some(root.as_path())
                && outcome
                    .job
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("job-"))
            {
                if let Some(handle) = process(outcome.helper_pid, &outcome.job.join("updater.exe"))?
                {
                    unsafe { WaitForSingleObject(handle.as_raw_handle(), 3000) };
                }
                if let Err(error) = fs::remove_dir_all(&outcome.job) {
                    tracing::warn!(error = %error, "Cannot clean completed update download");
                }
            }
            self.journal.clear_outcome()?;
            return Ok(outcome.error);
        }
        Ok(None)
    }
}
impl Prepared {
    pub fn launch(self, cancel: &cardo_runtime::task::CancellationToken) -> Result<()> {
        let service = &self.service;
        ensure!(!cancel.is_cancelled(), tr("task-cancelled"));
        let executable = std::env::current_exe()?;
        fs::copy(&executable, self.job.path().join("updater.exe"))?;
        for name in &service.config.helper_files {
            fs::copy(self.plan.target.join(name), self.job.path().join(name))?;
        }
        service
            .journal
            .save_pending(&serde_json::to_value(&self.plan)?)?;
        let spawn = Command::new(self.job.path().join("updater.exe"))
            .arg(&service.config.helper_argument)
            .creation_flags(0x08000000)
            .spawn();
        let mut child = match spawn {
            Ok(child) => child,
            Err(error) => {
                service.journal.clear_pending()?;
                return Err(error.into());
            }
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if self.job.path().join("ready").is_file() {
                break;
            }
            if child.try_wait()?.is_some() || Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                service.journal.clear_pending()?;
                bail!(tr("update-helper-failed"));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = self.job.keep();
        let _ = self.stage.keep();
        Ok(())
    }
}

fn process(id: u32, expected: &Path) -> Result<Option<OwnedHandle>> {
    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            id,
        )
    };
    if handle.is_null() {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(87) {
            return Ok(None);
        }
        return Err(error.into());
    }
    let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
    if unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) } == WAIT_OBJECT_0 {
        return Ok(None);
    }
    let mut name = vec![0u16; 32768];
    let mut len = name.len() as u32;
    ensure!(
        unsafe {
            QueryFullProcessImageNameW(handle.as_raw_handle(), 0, name.as_mut_ptr(), &mut len)
        } != 0,
        tr("update-helper-failed")
    );
    let actual = PathBuf::from(String::from_utf16(&name[..len as usize])?);
    let expected = match fs::canonicalize(expected) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if fs::canonicalize(actual)? != expected {
        return Ok(None);
    }
    Ok(Some(handle))
}
