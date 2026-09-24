use anyhow::Result;
use std::path::{Path, PathBuf};
use windows::{
    Win32::{
        Foundation::{ERROR_CANCELLED, HWND},
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize,
        },
        UI::Shell::{
            FOS_ALLOWMULTISELECT, FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, FileOpenDialog,
            FileSaveDialog, IFileDialog, IFileOpenDialog, IShellItem, SHCreateItemFromParsingName,
            SIGDN_FILESYSPATH,
        },
    },
    core::{HSTRING, Interface},
};
#[derive(Default)]
pub struct FileDialog {
    title: String,
    directory: Option<PathBuf>,
    filename: Option<String>,
}
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
impl FileDialog {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn set_directory(mut self, path: impl AsRef<Path>) -> Self {
        self.directory = Some(path.as_ref().into());
        self
    }
    pub fn set_file_name(mut self, name: impl Into<String>) -> Self {
        self.filename = Some(name.into());
        self
    }
    pub fn pick_file(self) -> Result<Option<PathBuf>> {
        Ok(self.run(false, false, false)?.and_then(|mut p| p.pop()))
    }
    pub fn pick_folder(self) -> Result<Option<PathBuf>> {
        Ok(self.run(false, true, false)?.and_then(|mut p| p.pop()))
    }
    pub fn save_file(self) -> Result<Option<PathBuf>> {
        Ok(self.run(true, false, false)?.and_then(|mut p| p.pop()))
    }
    pub fn pick_files(self) -> Result<Option<Vec<PathBuf>>> {
        self.run(false, false, true)
    }
    pub fn pick_folders(self) -> Result<Option<Vec<PathBuf>>> {
        self.run(false, true, true)
    }
    fn run(self, save: bool, folder: bool, multiple: bool) -> Result<Option<Vec<PathBuf>>> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            let _apartment = Apartment;
            let dialog: IFileDialog = CoCreateInstance(
                if save {
                    &FileSaveDialog
                } else {
                    &FileOpenDialog
                },
                None,
                CLSCTX_INPROC_SERVER,
            )?;
            let mut options = dialog.GetOptions()? | FOS_FORCEFILESYSTEM;
            if folder {
                options |= FOS_PICKFOLDERS;
            }
            if multiple {
                options |= FOS_ALLOWMULTISELECT;
            }
            dialog.SetOptions(options)?;
            if !self.title.is_empty() {
                dialog.SetTitle(&HSTRING::from(self.title))?;
            }
            if let Some(name) = self.filename {
                dialog.SetFileName(&HSTRING::from(name))?;
            }
            if let Some(path) = self.directory {
                let item: IShellItem =
                    SHCreateItemFromParsingName(&HSTRING::from(path.as_os_str()), None)?;
                dialog.SetFolder(&item)?;
            }
            if let Err(error) = dialog.Show(Some(HWND::default())) {
                if error.code() == windows::core::HRESULT::from_win32(ERROR_CANCELLED.0) {
                    return Ok(None);
                }
                return Err(error.into());
            }
            let mut result = Vec::new();
            if multiple {
                let items = dialog.cast::<IFileOpenDialog>()?.GetResults()?;
                for index in 0..items.GetCount()? {
                    result.push(item_path(&items.GetItemAt(index)?)?);
                }
            } else {
                result.push(item_path(&dialog.GetResult()?)?);
            }
            Ok(Some(result))
        }
    }
}
unsafe fn item_path(item: &IShellItem) -> Result<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    let raw = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
    let path = unsafe { std::ffi::OsString::from_wide(raw.as_wide()) };
    unsafe {
        CoTaskMemFree(Some(raw.0.cast()));
    }
    Ok(path.into())
}
