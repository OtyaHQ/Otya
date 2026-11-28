#![cfg(target_os = "linux")]
#![forbid(unsafe_code)]

//! Optional SQLite persistence for one operator-owned Otya Engine installation.
//!
//! This crate is intentionally separate from the core. It links to the host's
//! SQLite library and is not part of the default core/CLI dependency graph.
//! The operator must prepare a private data directory before opening a database.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use otya_engine_core::{SecureFsError, create_private_file, prepare_private_file};
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use thiserror::Error;

/// SQLite application ID identifying databases owned by Otya Engine ("OTYA").
pub const SQLITE_APPLICATION_ID: i64 = 0x4F54_5941;

/// Current metadata schema understood by this adapter.
pub const SQLITE_SCHEMA_VERSION: u32 = 1;

/// Stable identity recorded for the initial adapter metadata migration.
const INITIAL_MIGRATION_ID: &str = "otya-engine-storage-sqlite-initial-metadata-v1";

/// Default lock wait used when another connection is writing.
pub const DEFAULT_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Connection options for one operator-owned SQLite database.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SqliteOptions {
    /// Bounded wait for SQLite locks. Values from 100 ms through 120 s are allowed.
    pub busy_timeout: Duration,
}

impl Default for SqliteOptions {
    fn default() -> Self {
        Self {
            busy_timeout: DEFAULT_BUSY_TIMEOUT,
        }
    }
}

/// Result of running SQLite's built-in integrity check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegrityReport {
    /// SQLite result rows; a healthy database returns exactly ["ok"].
    pub messages: Vec<String>,
}

impl IntegrityReport {
    pub fn is_ok(&self) -> bool {
        self.messages.len() == 1 && self.messages[0].eq_ignore_ascii_case("ok")
    }
}

/// Errors from opening, identifying, migrating or using an Otya SQLite database.
#[derive(Debug, Error)]
pub enum SqliteStoreError {
    #[error("SQLite database path must be absolute: {0}")]
    PathNotAbsolute(PathBuf),
    #[error("SQLite database path must have a UTF-8 file name: {0}")]
    NonUtf8FileName(PathBuf),
    #[error("SQLite busy timeout must be between 100 ms and 120000 ms")]
    InvalidBusyTimeout,
    #[error("SQLite database is not an Otya Engine database: {0}")]
    UnclaimedDatabase(PathBuf),
    #[error("SQLite application_id {0} belongs to another application")]
    WrongApplicationId(i64),
    #[error("SQLite schema version {found} is newer than supported version {supported}")]
    UnsupportedSchemaVersion { found: i64, supported: u32 },
    #[error("Otya SQLite schema metadata is missing or inconsistent")]
    InvalidSchemaMetadata,
    #[error("SQLite could not enable WAL journal mode; returned mode was {0}")]
    WalUnavailable(String),
    #[error("secure filesystem operation failed: {0}")]
    SecureFilesystem(#[from] SecureFsError),
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// A connection to one private, operator-owned SQLite database.
///
/// rusqlite::Connection is not Sync. Use one store connection per worker or
/// protect one connection behind the host application's synchronization
/// primitive. WAL and SQLite's busy timeout coordinate separate connections;
/// they do not make a single connection safe for concurrent shared use.
pub struct SqliteStore {
    path: PathBuf,
    connection: Connection,
}

impl SqliteStore {
    /// Open or initialize a SQLite database at an absolute path.
    ///
    /// The parent directory must already exist and be owned by the operator.
    /// The parent is verified through no-follow directory descriptors, the
    /// database file is created with mode 0600 if missing, and an existing file
    /// must be an operator-owned regular file. Existing symlinks are rejected.
    ///
    /// SQLite is opened without CREATE and with SQLITE_OPEN_NOFOLLOW.
    /// Unknown databases are never silently claimed or modified.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteStoreError> {
        Self::open_with_options(path, SqliteOptions::default())
    }

    /// Open with an explicit bounded busy timeout.
    pub fn open_with_options(
        path: impl AsRef<Path>,
        options: SqliteOptions,
    ) -> Result<Self, SqliteStoreError> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err(SqliteStoreError::PathNotAbsolute(path.to_path_buf()));
        }

        if !(Duration::from_millis(100)..=Duration::from_secs(120)).contains(&options.busy_timeout)
        {
            return Err(SqliteStoreError::InvalidBusyTimeout);
        }

        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| SqliteStoreError::PathNotAbsolute(path.to_path_buf()))?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| SqliteStoreError::NonUtf8FileName(path.to_path_buf()))?;

        // O_EXCL prevents clobbering. An existing entry (including a symlink)
        // is returned as a typed error without opening or following the entry.
        match create_private_file(parent, file_name, &[]) {
            Ok(_) => {}
            Err(SecureFsError::FileAlreadyExists(_)) => {}
            Err(error) => return Err(error.into()),
        }
        prepare_private_file(parent, file_name)?;

        // Deliberately omit CREATE and URI flags. NOFOLLOW rejects a final
        // symlink even if the path changes between the secure check and open.
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_NOFOLLOW
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let mut connection = Connection::open_with_flags(path, flags)?;
        connection.busy_timeout(options.busy_timeout)?;

        // Set connection-local safety and durability defaults. WAL is enabled
        // only after database identity is checked and the first migration commits.
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA trusted_schema = OFF;
             PRAGMA synchronous = FULL;
             PRAGMA temp_store = MEMORY;",
        )?;

        initialize_or_verify_schema(&mut connection, path)?;

        let journal_mode: String =
            connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
        if !journal_mode.eq_ignore_ascii_case("wal") {
            return Err(SqliteStoreError::WalUnavailable(journal_mode));
        }

        Ok(Self {
            path: path.to_path_buf(),
            connection,
        })
    }

    /// Absolute path used to open this connection.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Schema version read from SQLite's user_version header.
    pub fn schema_version(&self) -> Result<u32, SqliteStoreError> {
        let version: i64 = self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        u32::try_from(version).map_err(|_| SqliteStoreError::InvalidSchemaMetadata)
    }

    /// Execute one write transaction, acquiring a write reservation first.
    /// Any callback error causes rollback when the transaction is dropped.
    pub fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&Transaction<'_>) -> rusqlite::Result<T>,
    ) -> Result<T, SqliteStoreError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let output = operation(&transaction)?;
        transaction.commit()?;
        Ok(output)
    }

    /// Run SQLite's full integrity check.
    pub fn integrity_check(&self) -> Result<IntegrityReport, SqliteStoreError> {
        let mut statement = self.connection.prepare("PRAGMA integrity_check")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(IntegrityReport { messages })
    }
}

fn initialize_or_verify_schema(
    connection: &mut Connection,
    path: &Path,
) -> Result<(), SqliteStoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Exclusive)?;
    let application_id: i64 =
        transaction.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    let schema_version: i64 = transaction.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if application_id == 0 && schema_version == 0 {
        let user_objects: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM sqlite_schema
             WHERE type IN ('table', 'index', 'trigger', 'view')
               AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )?;
        if user_objects != 0 {
            return Err(SqliteStoreError::UnclaimedDatabase(path.to_path_buf()));
        }

        transaction.execute_batch(
            "CREATE TABLE otya_engine_migrations (
                version INTEGER PRIMARY KEY CHECK (version > 0),
                migration_id TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
             );",
        )?;
        transaction.execute(
            "INSERT INTO otya_engine_migrations (version, migration_id)
             VALUES (?1, ?2)",
            (i64::from(SQLITE_SCHEMA_VERSION), INITIAL_MIGRATION_ID),
        )?;
        transaction.pragma_update(None, "application_id", SQLITE_APPLICATION_ID)?;
        transaction.pragma_update(None, "user_version", i64::from(SQLITE_SCHEMA_VERSION))?;
    } else {
        if application_id != SQLITE_APPLICATION_ID {
            return Err(SqliteStoreError::WrongApplicationId(application_id));
        }
        if schema_version > i64::from(SQLITE_SCHEMA_VERSION) {
            return Err(SqliteStoreError::UnsupportedSchemaVersion {
                found: schema_version,
                supported: SQLITE_SCHEMA_VERSION,
            });
        }
        if schema_version != i64::from(SQLITE_SCHEMA_VERSION) {
            return Err(SqliteStoreError::InvalidSchemaMetadata);
        }

        let migration_id: String = transaction
            .query_row(
                "SELECT migration_id FROM otya_engine_migrations WHERE version = ?1",
                [i64::from(SQLITE_SCHEMA_VERSION)],
                |row| row.get(0),
            )
            .map_err(|_| SqliteStoreError::InvalidSchemaMetadata)?;
        if migration_id != INITIAL_MIGRATION_ID {
            return Err(SqliteStoreError::InvalidSchemaMetadata);
        }
    }

    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn root_user() -> bool {
        rustix::process::geteuid().is_root()
    }

    fn temporary_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "otya-sqlite-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("test root should be created");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .expect("test root should be private");
        path.join("data")
    }

    fn create_data_dir(path: &Path) {
        fs::create_dir(path).expect("data directory should be created");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .expect("data directory should be private");
    }

    #[test]
    fn initializes_private_wal_database_and_metadata() {
        if root_user() {
            return;
        }
        let data_dir = temporary_root("new");
        create_data_dir(&data_dir);
        let database = data_dir.join("engine.db");

        let store = SqliteStore::open(&database).expect("database should be initialized");
        assert_eq!(store.path(), database);
        assert_eq!(store.schema_version().expect("version should read"), 1);
        assert!(
            store
                .integrity_check()
                .expect("integrity check should run")
                .is_ok()
        );
        assert_eq!(
            fs::metadata(&database)
                .expect("database metadata should be available")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );

        let journal_mode: String = store
            .connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal mode should be readable");
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
        let _ = fs::remove_dir_all(data_dir.parent().expect("test parent should exist"));
    }

    #[test]
    fn reopens_existing_database_and_preserves_schema_identity() {
        if root_user() {
            return;
        }
        let data_dir = temporary_root("reopen");
        create_data_dir(&data_dir);
        let database = data_dir.join("engine.db");

        {
            let store = SqliteStore::open(&database).expect("database should initialize");
            let application_id: i64 = store
                .connection
                .query_row("PRAGMA application_id", [], |row| row.get(0))
                .expect("application id should be available");
            assert_eq!(application_id, SQLITE_APPLICATION_ID);
            assert_eq!(store.schema_version().expect("schema should read"), 1);
        }

        fs::set_permissions(&database, fs::Permissions::from_mode(0o444))
            .expect("database should be made read-only for hardening test");
        let reopened = SqliteStore::open(&database).expect("existing database should reopen");
        assert_eq!(reopened.schema_version().expect("schema should read"), 1);
        assert_eq!(
            fs::metadata(&database)
                .expect("database metadata should be available")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        drop(reopened);
        let _ = fs::remove_dir_all(data_dir.parent().expect("test parent should exist"));
    }

    #[test]
    fn commits_successful_transactions_and_rolls_back_errors() {
        if root_user() {
            return;
        }
        let data_dir = temporary_root("transactions");
        create_data_dir(&data_dir);
        let database = data_dir.join("engine.db");
        let mut store = SqliteStore::open(&database).expect("database should open");

        store
            .transaction(|transaction| {
                transaction.execute_batch(
                    "CREATE TABLE sample_records (
                        id INTEGER PRIMARY KEY,
                        value TEXT NOT NULL
                     );
                     INSERT INTO sample_records (value) VALUES ('committed');",
                )
            })
            .expect("successful transaction should commit");

        let aborted: Result<(), _> = store.transaction(|transaction| {
            transaction.execute(
                "INSERT INTO sample_records (value) VALUES (?1)",
                ["rolled-back"],
            )?;
            Err(rusqlite::Error::InvalidQuery)
        });
        assert!(aborted.is_err());

        let count: i64 = store
            .transaction(|transaction| {
                transaction.query_row("SELECT COUNT(*) FROM sample_records", [], |row| row.get(0))
            })
            .expect("read transaction should complete");
        assert_eq!(count, 1);
        let _ = fs::remove_dir_all(data_dir.parent().expect("test parent should exist"));
    }

    #[test]
    fn refuses_to_claim_an_existing_non_otya_database() {
        if root_user() {
            return;
        }
        let data_dir = temporary_root("unclaimed");
        create_data_dir(&data_dir);
        let database = data_dir.join("existing.db");
        let foreign = Connection::open(&database).expect("foreign database should open");
        foreign
            .execute_batch("CREATE TABLE third_party_data (id INTEGER PRIMARY KEY);")
            .expect("foreign table should be created");
        drop(foreign);
        let original_contents = fs::read(&database).expect("foreign database should be readable");

        assert!(matches!(
            SqliteStore::open(&database),
            Err(SqliteStoreError::UnclaimedDatabase(_))
        ));
        assert_eq!(
            fs::read(&database).expect("foreign database should remain readable"),
            original_contents
        );
        assert_eq!(
            fs::metadata(&database)
                .expect("foreign database metadata should be available")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let _ = fs::remove_dir_all(data_dir.parent().expect("test parent should exist"));
    }

    #[test]
    fn refuses_symlink_database_files() {
        if root_user() {
            return;
        }
        let data_dir = temporary_root("symlink");
        create_data_dir(&data_dir);
        let target = data_dir.join("actual.db");
        drop(Connection::open(&target).expect("target database should be created"));
        let link = data_dir.join("linked.db");
        symlink(&target, &link).expect("symlink should be created");

        assert!(SqliteStore::open(&link).is_err());
        let _ = fs::remove_dir_all(data_dir.parent().expect("test parent should exist"));
    }

    #[test]
    fn rejects_relative_paths_and_unbounded_busy_timeouts() {
        assert!(matches!(
            SqliteStore::open("relative/engine.db"),
            Err(SqliteStoreError::PathNotAbsolute(_))
        ));
        if root_user() {
            return;
        }
        let data_dir = temporary_root("timeout");
        create_data_dir(&data_dir);
        let database = data_dir.join("engine.db");

        assert!(matches!(
            SqliteStore::open_with_options(
                &database,
                SqliteOptions {
                    busy_timeout: Duration::from_millis(1),
                }
            ),
            Err(SqliteStoreError::InvalidBusyTimeout)
        ));
        let _ = fs::remove_dir_all(data_dir.parent().expect("test parent should exist"));
    }
}
