//! Human-editable TOML configuration with optimistic concurrency and atomic saves.
use crate::storage::{Store, lock_file};
use anyhow::{Context, Result, ensure};
use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, TableLike};

pub struct Snapshot<T> {
    path: PathBuf,
    source: Option<String>,
    value: T,
}

impl<T: Serialize + DeserializeOwned> Snapshot<T> {
    /// Only a missing file uses `default`; malformed or unreadable files are errors.
    pub fn load(path: impl Into<PathBuf>, default: impl FnOnce() -> Result<T>) -> Result<Self> {
        let path = path.into();
        let source = read(&path)?;
        let value = match &source {
            Some(source) => toml::from_str(source)
                .with_context(|| format!("Invalid TOML in {}", path.display()))?,
            None => default()?,
        };
        Ok(Self {
            path,
            source,
            value,
        })
    }

    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn exists(&self) -> bool {
        self.source.is_some()
    }

    /// Callers validate product semantics before saving. A stale snapshot never
    /// overwrites another application save or a previously observed external edit.
    pub fn save(&mut self, value: T) -> Result<()> {
        let parent = self
            .path
            .parent()
            .context("Configuration requires a parent directory")?;
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create {}", parent.display()))?;
        let _lock = lock_file(&self.path.with_extension("toml.lock"))?;
        ensure!(
            read(&self.path)? == self.source,
            "Configuration changed outside this session: {}. Reload before saving.",
            self.path.display()
        );
        let new = toml::to_string_pretty(&value)
            .with_context(|| format!("Cannot serialize configuration {}", self.path.display()))?;
        let next = if let Some(source) = &self.source {
            let before = toml::to_string_pretty(&self.value)?.parse::<DocumentMut>()?;
            let after = new.parse::<DocumentMut>()?;
            let mut document = source.parse::<DocumentMut>()?;
            merge(document.as_table_mut(), before.as_table(), after.as_table());
            document.to_string()
        } else {
            new
        };
        // Parsing again also ensures the merged representation remains type-correct.
        let parsed: T = toml::from_str(&next).with_context(|| {
            format!("Invalid configuration after merge: {}", self.path.display())
        })?;
        let files = Store::at(parent);
        if let Some(source) = &self.source {
            let backup = self.path.with_extension("toml.bak");
            let name = backup
                .file_name()
                .and_then(|n| n.to_str())
                .context("Invalid configuration filename")?;
            files.write(name, source.as_bytes())?;
        }
        let name = self
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid configuration filename")?;
        files.write(name, next.as_bytes())?;
        self.source = Some(next);
        self.value = parsed;
        Ok(())
    }
}

fn read(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("Cannot read {}", path.display())),
    }
}

fn merge(target: &mut dyn TableLike, before: &dyn TableLike, after: &dyn TableLike) {
    for (key, _) in before.iter() {
        if !after.contains_key(key) {
            target.remove(key);
        }
    }
    for (key, next) in after.iter() {
        let previous = before.get(key);
        if previous.is_some_and(|item| item.to_string() == next.to_string()) {
            continue;
        }
        if let (Some(current), Some(old), Some(new)) = (
            target.get_mut(key).and_then(Item::as_table_like_mut),
            previous.and_then(Item::as_table_like),
            next.as_table_like(),
        ) {
            merge(current, old, new);
        } else {
            let mut next = next.clone();
            if let (Some(old), Some(new)) = (
                target.get(key).and_then(Item::as_value),
                next.as_value_mut(),
            ) {
                *new.decor_mut() = old.decor().clone();
            }
            target.insert(key, next);
        }
    }
}
