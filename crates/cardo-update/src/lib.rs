#[cfg(windows)]
mod apply;
#[cfg(windows)]
mod download;
mod release;
use anyhow::{Result, ensure};
#[cfg(windows)]
pub use apply::{Prepared, Target};
#[cfg(windows)]
pub use download::{Phase, Progress};
pub use release::Status;
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Child,
    sync::Arc,
};
#[derive(Clone)]
pub struct UpdateConfig {
    pub application_id: String,
    pub version: String,
    pub repository: Option<String>,
    pub installer: String,
    pub portable: String,
    pub checksums: String,
    pub archive_root: String,
    pub files: Vec<String>,
    pub executable: String,
    pub helper_files: Vec<String>,
    pub helper_argument: String,
    pub jobs: PathBuf,
    pub max_package_bytes: u64,
    pub stage_prefix: String,
}
pub trait UpdateJournal: Send + Sync {
    fn pending(&self) -> Result<Option<Value>>;
    fn save_pending(&self, value: &Value) -> Result<()>;
    fn clear_pending(&self) -> Result<()>;
    fn outcome(&self) -> Result<Option<Value>>;
    fn finish(&self, value: &Value) -> Result<()>;
    fn clear_outcome(&self) -> Result<()>;
}
pub trait InstallAdapter: Send + Sync {
    fn registration(&self, target: &Path) -> Result<Option<Value>>;
    fn backup_files(&self, target: &Path, registration: &Value) -> Result<Vec<String>>;
    fn restore(&self, target: &Path, registration: &Value) -> Result<()>;
    fn spawn(&self, package: &Path, target: &Path) -> Result<Child>;
    fn verify(&self, target: &Path, version: &str) -> Result<()>;
}
#[derive(Clone)]
pub struct UpdateService {
    pub config: UpdateConfig,
    pub(crate) journal: Arc<dyn UpdateJournal>,
    pub(crate) installer: Arc<dyn InstallAdapter>,
}
impl UpdateService {
    pub fn new(
        config: UpdateConfig,
        journal: Arc<dyn UpdateJournal>,
        installer: Arc<dyn InstallAdapter>,
    ) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            journal,
            installer,
        })
    }
}

impl UpdateConfig {
    fn validate(&self) -> Result<()> {
        let relative = |name: &str| {
            !name.is_empty()
                && Path::new(name)
                    .components()
                    .all(|p| matches!(p, std::path::Component::Normal(_)))
        };
        let filename = |name: &str| relative(name) && Path::new(name).components().count() == 1;
        ensure!(
            !self.application_id.is_empty(),
            "Update application identity is required"
        );
        semver::Version::parse(&self.version)?;
        ensure!(
            self.jobs.is_absolute() && self.max_package_bytes > 0,
            "Invalid update storage or size limit"
        );
        ensure!(
            filename(&self.installer)
                && filename(&self.portable)
                && filename(&self.checksums)
                && filename(&self.archive_root)
                && filename(&self.stage_prefix),
            "Invalid update package name"
        );
        ensure!(
            relative(&self.executable) && self.files.contains(&self.executable),
            "Update executable must be in the file manifest"
        );
        let mut seen = std::collections::BTreeSet::new();
        ensure!(
            self.files
                .iter()
                .all(|f| relative(f) && seen.insert(f.to_lowercase())),
            "Invalid or duplicate update manifest entry"
        );
        ensure!(
            self.helper_files
                .iter()
                .all(|f| filename(f) && self.files.contains(f)),
            "Invalid updater runtime file"
        );
        if let Some(repository) = self.repository.as_deref().filter(|v| !v.is_empty()) {
            let parts: Vec<_> = repository.split('/').collect();
            ensure!(
                parts.len() == 2
                    && parts.iter().all(|p| !p.is_empty()
                        && p.bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))),
                "Invalid release repository"
            );
        }
        Ok(())
    }
}
