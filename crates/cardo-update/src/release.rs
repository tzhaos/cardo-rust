use crate::UpdateService;
use anyhow::{Context, Result, bail};
use cardo_runtime::messages::tr;
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use std::time::Duration;
#[derive(Clone)]
pub enum Status {
    Checking,
    Unconfigured,
    Current,
    Downloading,
    Available {
        version: String,
        download: String,
        portable: String,
        checksums: String,
        installer_size: u64,
        portable_size: u64,
        release: String,
    },
    Failed(String),
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
}

impl UpdateService {
    pub fn check(&self) -> Result<Status> {
        let Some(repository) = self
            .config
            .repository
            .as_deref()
            .filter(|value| !value.is_empty())
        else {
            return Ok(Status::Unconfigured);
        };
        let client = Client::builder()
            .user_agent(format!(
                "{}/{}",
                self.config.application_id, self.config.version
            ))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(25))
            .build()?;
        let response = client
            .get(format!(
                "https://api.github.com/repos/{repository}/releases/latest"
            ))
            .header("Accept", "application/vnd.github+json")
            .send()?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            bail!(tr("update-no-release"));
        }
        let release: Release = response.error_for_status()?.json()?;
        let version = Version::parse(
            release
                .tag_name
                .strip_prefix('v')
                .unwrap_or(&release.tag_name),
        )
        .context(tr("update-version-invalid"))?;
        if release.draft || release.prerelease || !version.pre.is_empty() {
            bail!(tr("update-no-release"));
        }
        if version <= Version::parse(&self.config.version)? {
            return Ok(Status::Current);
        }
        let base = format!("https://github.com/{repository}/releases/");
        let download_asset = |name: &str| -> Result<String> {
            let asset = release
                .assets
                .iter()
                .find(|asset| asset.name == name)
                .context(tr("update-asset-missing"))?;
            let mut expected = reqwest::Url::parse(&format!("{base}download/"))?;
            expected
                .path_segments_mut()
                .map_err(|_| anyhow::anyhow!(tr("update-url-invalid")))?
                .pop_if_empty()
                .push(&release.tag_name)
                .push(name);
            if reqwest::Url::parse(&asset.browser_download_url)? != expected {
                bail!(tr("update-url-invalid"));
            }
            Ok(expected.into())
        };
        let download = download_asset(&self.config.installer)?;
        let portable = download_asset(&self.config.portable)?;
        let checksums = download_asset(&self.config.checksums)?;
        let size = |name: &str| {
            release
                .assets
                .iter()
                .find(|asset| asset.name == name)
                .map(|asset| asset.size)
                .unwrap_or(0)
        };
        let installer_size = size(&self.config.installer);
        let portable_size = size(&self.config.portable);
        let mut page = reqwest::Url::parse(&format!("{base}tag/"))?;
        page.path_segments_mut()
            .map_err(|_| anyhow::anyhow!(tr("update-url-invalid")))?
            .pop_if_empty()
            .push(&release.tag_name);
        Ok(Status::Available {
            version: version.to_string(),
            download,
            portable,
            checksums,
            installer_size,
            portable_size,
            release: page.into(),
        })
    }
}
