#![cfg(target_os = "linux")]
#![forbid(unsafe_code)]

//! Optional SQLite persistence for one operator-owned Otya Engine installation.
//!
//! This crate is intentionally separate from the core. It links to the host's
//! SQLite library and is not part of the default core/CLI dependency graph.
//! The operator must prepare a private data directory before opening a database.

use std::{
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use otya_engine_core::{
    MAX_STORAGE_KEY_BYTES, MAX_STORAGE_NAMESPACE_BYTES, MAX_STORAGE_SCAN_LIMIT,
    MAX_STORAGE_VALUE_BYTES, SecureFsError, Storage, StorageEntry, StorageError,
    StorageTransaction, create_private_file, prepare_private_file,
};
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
};
use thiserror::Error;

/// SQLite application ID identifying databases owned by Otya Engine ("OTYA").
pub const SQLITE_APPLICATION_ID: i64 = 0x4F54_5941;

/// Current metadata and key/value schema understood by this adapter.
pub const SQLITE_SCHEMA_VERSION: u32 = 2;

/// Stable identity recorded for the initial metadata migration.
const INITIAL_MIGRATION_ID: &str = "otya-engine-storage-sqlite-initial-metadata-v1";

/// Stable identity recorded for the namespaced key/value migration.
const KEY_VALUE_MIGRATION_ID: &str = "otya-engine-storage-sqlite-namespaced-kv-v2";

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
    #[error("SQLite connection lock was poisoned")]
    ConnectionPoisoned,
    #[error("secure filesystem operation failed: {0}")]
    SecureFilesystem(#[from] SecureFsError),
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// A connection to one private, operator-owned SQLite database.
///
/// The connection is protected by a mutex so the backend satisfies the core
/// Storage Send + Sync contract. A transaction holds that lock for its whole
/// lifetime. Use one SqliteStore per worker when parallel read/write throughput
/// is needed; SQLite WAL and busy timeout coordinate separate connections.
pub struct SqliteStore {
    path: PathBuf,
    connection: Mutex<Connection>,
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

        // These are connection-local safety/durability defaults.
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
            connection: Mutex::new(connection),
        })
    }

    /// Absolute path used to open this connection.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Schema version read from SQLite's user_version header.
    pub fn schema_version(&self) -> Result<u32, SqliteStoreError> {
        let connection = self.lock_connection()?;
        let version: i64 =
            connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        u32::try_from(version).map_err(|_| SqliteStoreError::InvalidSchemaMetadata)
    }

    /// Execute a typed rusqlite transaction as a convenience for Rust hosts.
    ///
    /// The connection lock is held for the transaction lifetime. On callback
    /// error, the rusqlite transaction is dropped and rolled back.
    pub fn transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> rusqlite::Result<T>,
    ) -> Result<T, SqliteStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let output = operation(&transaction)?;
        transaction.commit()?;
        Ok(output)
    }

    /// Run SQLite's full integrity check without changing application records.
    pub fn integrity_check(&self) -> Result<IntegrityReport, SqliteStoreError> {
        let connection = self.lock_connection()?;
        let mut statement = connection.prepare("PRAGMA integrity_check")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(IntegrityReport { messages })
    }

    fn lock_connection(&self) -> Result<MutexGuard<'_, Connection>, SqliteStoreError> {
        self.connection
            .lock()
            .map_err(|_| SqliteStoreError::ConnectionPoisoned)
    }
}

impl Storage for SqliteStore {
    fn begin(&self) -> Result<Box<dyn StorageTransaction + '_>, StorageError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| StorageError::Backend("SQLite connection lock was poisoned".into()))?;
        connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(storage_backend_error)?;

        Ok(Box::new(SqliteStorageTransaction {
            connection,
            completed: false,
        }))
    }

    fn health_check(&self) -> Result<(), StorageError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| StorageError::Backend("SQLite connection lock was poisoned".into()))?;
        let message: String = connection
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .map_err(storage_backend_error)?;
        if message.eq_ignore_ascii_case("ok") {
            Ok(())
        } else {
            Err(StorageError::Backend(format!(
                "SQLite quick_check returned {message}"
            )))
        }
    }
}

struct SqliteStorageTransaction<'connection> {
    connection: MutexGuard<'connection, Connection>,
    completed: bool,
}

impl StorageTransaction for SqliteStorageTransaction<'_> {
    fn get(
        &mut self,
        namespace: &str,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, StorageError> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        self.connection
            .query_row(
                "SELECT value FROM otya_engine_kv WHERE namespace = ?1 AND key = ?2",
                params![namespace, key],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_backend_error)
    }

    fn put(
        &mut self,
        namespace: &str,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), StorageError> {
        validate_namespace(namespace)?;
        validate_key(key)?;
        validate_value(value)?;

        self.connection
            .execute(
                "INSERT INTO otya_engine_kv (namespace, key, value)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(namespace, key) DO UPDATE SET value = excluded.value",
                params![namespace, key, value],
            )
            .map_err(storage_backend_error)?;
        Ok(())
    }

    fn delete(&mut self, namespace: &str, key: &[u8]) -> Result<bool, StorageError> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let changed = self
            .connection
            .execute(
                "DELETE FROM otya_engine_kv WHERE namespace = ?1 AND key = ?2",
                params![namespace, key],
            )
            .map_err(storage_backend_error)?;
        Ok(changed == 1)
    }

    fn scan_prefix(
        &mut self,
        namespace: &str,
        prefix: &[u8],
        limit: usize,
    ) -> Result<Vec<StorageEntry>, StorageError> {
        validate_namespace(namespace)?;
        validate_key(prefix)?;
        if !(1..=MAX_STORAGE_SCAN_LIMIT).contains(&limit) {
            return Err(StorageError::InvalidScanLimit);
        }

        let prefix_len = i64::try_from(prefix.len())
            .map_err(|_| StorageError::Backend("prefix length overflow".into()))?;
        let row_limit = i64::try_from(limit)
            .map_err(|_| StorageError::InvalidScanLimit)?;
        let mut statement = self.connection.prepare(
            "SELECT key, value
             FROM otya_engine_kv
             WHERE namespace = ?1 AND substr(key, 1, ?2) = ?3
             ORDER BY key ASC
             LIMIT ?4",
        )
        .map_err(storage_backend_error)?;
        let rows = statement
            .query_map(params![namespace, prefix_len, prefix, row_limit], |row| {
                Ok(StorageEntry {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            })
            .map_err(storage_backend_error)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(storage_backend_error)?);
        }
        Ok(entries)
    }

    fn commit(mut self: Box<Self>) -> Result<(), StorageError> {
        self.connection
            .execute_batch("COMMIT")
            .map_err(storage_backend_error)?;
        self.completed = true;
        Ok(())
    }

    fn rollback(mut self: Box<Self>) -> Result<(), StorageError> {
        self.connection
            .execute_batch("ROLLBACK")
            .map_err(storage_backend_error)?;
        self.completed = true;
        Ok(())
    }
}

impl Drop for SqliteStorageTransaction<'_> {
    fn drop(&mut self) {
        if !self.completed {
            let _ = self.connection.execute_batch("ROLLBACK");
        }
    }
}

fn validate_namespace(namespace: &str) -> Result<(), StorageError> {
    let bytes = namespace.as_bytes();
    if bytes.is_empty()
        || bytes.len() > MAX_STORAGE_NAMESPACE_BYTES
        || !bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-')
        })
    {
        return Err(StorageError::InvalidNamespace(namespace.to_owned()));
    }
    Ok(())
}

fn validate_key(key: &[u8]) -> Result<(), StorageError> {
    if key.len() > MAX_STORAGE_KEY_BYTES {
        return Err(StorageError::KeyTooLarge);
    }
    Ok(())
}

fn validate_value(value: &[u8]) -> Result<(), StorageError> {
    if value.len() > MAX_STORAGE_VALUE_BYTES {
        return Err(StorageError::ValueTooLarge);
    }
    Ok(())
}

fn storage_backend_error(error: rusqlite::Error) -> StorageError {
    StorageError::Backend(error.to_string())
}

fn initialize_or_verify_schema(
    connection: &mut Connection,
    path: &Path,
) -> Result<(), SqliteStoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Exclusive)?;
    let application_id: i64 =
        transaction.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    let mut schema_version: i64 =
        transaction.query_row("PRAGMA user_version", [], |row| row.get(0))?;

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
            (1_i64, INITIAL_MIGRATION_ID),
        )?;
        transaction.pragma_update(None, "application_id", SQLITE_APPLICATION_ID)?;
        transaction.pragma_update(None, "user_version", 1_i64)?;
        schema_version = 1;
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
        if schema_version < 1 {
            return Err(SqliteStoreError::InvalidSchemaMetadata);
        }

        let migration_id: String = transaction
            .query_row(
                "SELECT migration_id FROM otya_engine_migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|_| SqliteStoreError::InvalidSchemaMetadata)?;
        if migration_id != INITIAL_MIGRATION_ID {
            return Err(SqliteStoreError::InvalidSchemaMetadata);
        }
    }

    if schema_version == 1 {
        transaction.execute_batch(
            "CREATE TABLE otya_engine_kv (
                namespace TEXT NOT NULL
                    CHECK (
                        length(namespace) BETWEEN 1 AND 128
                        AND namespace NOT GLOB '*[^a-z0-9._-]*'
                    ),
                key BLOB NOT NULL CHECK (length(key) <= 4096),
                value BLOB NOT NULL CHECK (length(value) <= 16777216),
                PRIMARY KEY (namespace, key)
             ) WITHOUT ROWID;",
        )?;
        transaction.execute(
            "INSERT INTO otya_engine_migrations (version, migration_id)
             VALUES (?1, ?2)",
            (2_i64, KEY_VALUE_MIGRATION_ID),
        )?;
        transaction.pragma_update(
            None,
            "user_version",
            i64::from(SQLITE_SCHEMA_VERSION),
        )?;
        schema_version = i64::from(SQLITE_SCHEMA_VERSION);
    }

    if schema_version == i64::from(SQLITE_SCHEMA_VERSION) {
        let migration_id: String = transaction
            .query_row(
                "SELECT migration_id FROM otya_engine_migrations WHERE version = ?1",
                [i64::from(SQLITE_SCHEMA_VERSION)],
                |row| row.get(0),
            )
            .map_err(|_| SqliteStoreError::InvalidSchemaMetadata)?;
        if migration_id != KEY_VALUE_MIGRATION_ID {
            return Err(SqliteStoreError::InvalidSchemaMetadata);
        }

        let kv_table_count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM sqlite_schema
             WHERE type = 'table' AND name = 'otya_engine_kv'",
            [],
            |row| row.get(0),
        )?;
        if kv_table_count != 1 {
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

    use otya_engine_core::{Storage, StorageError, StorageTransaction};
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
        assert_eq!(store.schema_version().expect("version should read"), 2);
        assert!(store.integrity_check().expect("integrity check should run").is_ok());
        assert!(store.health_check().is_ok());
        assert_eq!(
            fs::metadata(&database)
                .expect("database metadata should be available")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );

        let journal_mode: String = store
            .lock_connection()
            .expect("connection should lock")
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal mode should be readable");
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");

        let mut transaction = store.begin().expect("storage transaction should begin");
        transaction
            .put("core", b"first", b"value")
            .expect("namespaced value should be written");
        transaction.commit().expect("storage transaction should commit");

        let mut read = store.begin().expect("read transaction should begin");
        assert_eq!(
            read.get("core", b"first").expect("value should be readable"),
            Some(b"value".to_vec())
        );
        read.commit().expect("read transaction should commit");
        drop(store);
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
                .lock_connection()
                .expect("connection should lock")
                .query_row("PRAGMA application_id", [], |row| row.get(0))
                .expect("application id should be available");
            assert_eq!(application_id, SQLITE_APPLICATION_ID);
            assert_eq!(store.schema_version().expect("schema should read"), 2);
        }

        fs::set_permissions(&database, fs::Permissions::from_mode(0o444))
            .expect("database should be made read-only for hardening test");
        let reopened = SqliteStore::open(&database).expect("existing database should reopen");
        assert_eq!(reopened.schema_version().expect("schema should read"), 2);
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
    fn storage_contract_supports_prefix_scans_and_validation() {
        if root_user() {
            return;
        }
        let data_dir = temporary_root("storage-contract");
        create_data_dir(&data_dir);
        let store = SqliteStore::open(data_dir.join("engine.db"))
            .expect("database should initialize");

        let mut transaction = store.begin().expect("transaction should begin");
        transaction.put("media", b"track/02", b"two").expect("put should work");
        transaction.put("media", b"track/01", b"one").expect("put should work");
        transaction.put("media", b"album/01", b"album").expect("put should work");
        transaction.commit().expect("transaction should commit");

        let mut scan = store.begin().expect("scan transaction should begin");
        let entries = scan
            .scan_prefix("media", b"track/", 10)
            .expect("prefix scan should work");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, b"track/01");
        assert_eq!(entries[1].key, b"track/02");
        assert!(scan.delete("media", b"track/02").expect("delete should work"));
        assert!(!scan.delete("media", b"missing").expect("delete should work"));
        scan.commit().expect("scan transaction should commit");

        let mut invalid = store.begin().expect("transaction should begin");
        assert!(matches!(
            invalid.put("BadNamespace", b"key", b"value"),
            Err(StorageError::InvalidNamespace(_))
        ));
        assert!(matches!(
            invalid.scan_prefix("media", b"", 0),
            Err(StorageError::InvalidScanLimit)
        ));
        let big_key = vec![0; MAX_STORAGE_KEY_BYTES + 1];
        assert_eq!(
            invalid.put("media", &big_key, b"value"),
            Err(StorageError::KeyTooLarge)
        );
        let big_value = vec![0; MAX_STORAGE_VALUE_BYTES + 1];
        assert_eq!(
            invalid.put("media", b"key", &big_value),
            Err(StorageError::ValueTooLarge)
        );
        invalid.rollback().expect("invalid-operation transaction should roll back");

        drop(store);
        let _ = fs::remove_dir_all(data_dir.parent().expect("test parent should exist"));
    }

    #[test]
    fn dropping_storage_transaction_rolls_back_uncommitted_writes() {
        if root_user() {
            return;
        }
        let data_dir = temporary_root("drop-rollback");
        create_data_dir(&data_dir);
        let store = SqliteStore::open(data_dir.join("engine.db"))
            .expect("database should initialize");

        {
            let mut transaction = store.begin().expect("transaction should begin");
            transaction.put("module", b"key", b"uncommitted").expect("put should work");
        }

        let mut check = store.begin().expect("check transaction should begin");
        assert_eq!(
            check.get("module", b"key").expect("get should work"),
            None
        );
        check.commit().expect("check transaction should commit");

        drop(store);
        let _ = fs::remove_dir_all(data_dir.parent().expect("test parent should exist"));
    }

    #[test]
    fn commits_successful_rusqlite_transactions_and_rolls_back_errors() {
        if root_user() {
            return;
        }
        let data_dir = temporary_root("transactions");
        create_data_dir(&data_dir);
        let mut store = SqliteStore::open(data_dir.join("engine.db"))
            .expect("database should open");

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
        drop(store);
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
