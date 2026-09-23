//! SQLite for local transactional state. Products own their schema and queries.
use anyhow::{Context, Result, ensure};
pub use rusqlite;
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone)]
pub struct Database {
    path: PathBuf,
    application_id: i32,
    version: i32,
}

impl Database {
    /// Initialize an empty database or open this exact schema version.
    /// Existing databases are never converted, rebuilt, or reset.
    pub fn open(
        path: impl Into<PathBuf>,
        application_id: i32,
        version: i32,
        schema: &str,
    ) -> Result<Self> {
        ensure!(
            application_id != 0 && version > 0,
            "Database identity and schema version are required"
        );
        let database = Self {
            path: path.into(),
            application_id,
            version,
        };
        database
            .initialize(schema)
            .with_context(|| format!("Cannot initialize database {}", database.path.display()))?;
        Ok(database)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn initialize(&self, schema: &str) -> Result<()> {
        let parent = self
            .path
            .parent()
            .context("Database requires a parent directory")?;
        std::fs::create_dir_all(parent)?;
        let _lock = crate::storage::lock_file(&self.path.with_extension("sqlite3.schema.lock"))?;
        let mut connection = self.connect(true)?;
        let id: i32 = connection.pragma_query_value(None, "application_id", |r| r.get(0))?;
        let version: i32 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if id == 0 {
            let tables: i64 =
                connection.query_row("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))?;
            ensure!(
                version == 0 && tables == 0,
                "Unrecognized database ownership"
            );
        } else {
            ensure!(
                id == self.application_id,
                "Database belongs to another application"
            );
            ensure!(
                version == self.version,
                "Unsupported database schema {version}; expected {}",
                self.version
            );
        }
        let mode: String =
            connection.pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get(0))?;
        ensure!(
            mode.eq_ignore_ascii_case("wal"),
            "Database does not support WAL on this filesystem"
        );
        if id == 0 {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(schema)?;
            transaction.pragma_update(None, "user_version", self.version)?;
            transaction.pragma_update(None, "application_id", self.application_id)?;
            transaction.commit()?;
        }
        Ok(())
    }

    fn connect(&self, create: bool) -> Result<Connection> {
        let mut flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        if create {
            flags |= OpenFlags::SQLITE_OPEN_CREATE;
        }
        let connection = Connection::open_with_flags(&self.path, flags)
            .with_context(|| format!("Cannot open database {}", self.path.display()))?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "synchronous", "FULL")?;
        if !create {
            let id: i32 = connection.pragma_query_value(None, "application_id", |r| r.get(0))?;
            let version: i32 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
            ensure!(
                id == self.application_id && version == self.version,
                "Database identity/schema changed: {}",
                self.path.display()
            );
        }
        Ok(connection)
    }

    /// Run on a worker thread. No connection or transaction is shared across threads.
    pub fn read<T>(&self, operation: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        (|| operation(&self.connect(false)?))()
            .with_context(|| format!("Cannot read database {}", self.path.display()))
    }

    /// A closure error rolls back the entire transaction. Keep external I/O outside it.
    pub fn write<T>(&self, operation: impl FnOnce(&Transaction<'_>) -> Result<T>) -> Result<T> {
        (|| -> Result<T> {
            let mut connection = self.connect(false)?;
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let value = operation(&transaction)?;
            transaction.commit()?;
            Ok(value)
        })()
        .with_context(|| format!("Cannot write database {}", self.path.display()))
    }

    pub fn backup(&self, destination: &Path) -> Result<()> {
        self.read(|connection| Self::backup_connection(connection, destination))
    }

    fn backup_connection(connection: &Connection, destination: &Path) -> Result<()> {
        let parent = destination
            .parent()
            .context("Backup requires a parent directory")?;
        let temporary = tempfile::NamedTempFile::new_in(parent)?;
        connection
            .backup(rusqlite::MAIN_DB, temporary.path(), None)
            .with_context(|| format!("Cannot back up database to {}", destination.display()))?;
        temporary.as_file().sync_all()?;
        // Never overwrite an existing backup or the source database.
        temporary
            .persist_noclobber(destination)
            .with_context(|| format!("Cannot publish database backup {}", destination.display()))?;
        Ok(())
    }

    pub fn check_integrity(&self) -> Result<()> {
        self.read(|connection| {
            let mut statement = connection.prepare("PRAGMA integrity_check")?;
            let messages = statement
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            ensure!(
                messages == ["ok"],
                "Database integrity check failed: {}",
                messages.join("; ")
            );
            Ok(())
        })
    }
}
