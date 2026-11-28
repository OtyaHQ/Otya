//! Backend-neutral storage contracts shared by optional adapters.

use thiserror::Error;

/// Maximum UTF-8 byte length of a storage namespace.
pub const MAX_STORAGE_NAMESPACE_BYTES: usize = 128;
/// Maximum key length in bytes.
pub const MAX_STORAGE_KEY_BYTES: usize = 4_096;
/// Maximum value length in bytes (16 MiB).
pub const MAX_STORAGE_VALUE_BYTES: usize = 16 * 1024 * 1024;
/// Maximum number of entries returned by one prefix scan.
pub const MAX_STORAGE_SCAN_LIMIT: usize = 10_000;

/// One opaque value returned by a storage prefix scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

/// Common errors returned by every storage adapter.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StorageError {
    #[error(
        "invalid storage namespace {0:?}; use 1-128 lowercase ASCII letters, digits, dots, underscores or hyphens"
    )]
    InvalidNamespace(String),
    #[error("storage key exceeds the 4096-byte limit")]
    KeyTooLarge,
    #[error("storage value exceeds the 16 MiB limit")]
    ValueTooLarge,
    #[error("storage prefix scan limit must be between 1 and 10000")]
    InvalidScanLimit,
    #[error("storage backend error: {0}")]
    Backend(String),
}

/// Backend-neutral storage for one operator-owned installation.
///
/// This is a generic Rust trait rather than a trait object because its
/// transaction handle borrows the store. A host can keep one store per worker
/// or protect access using its own synchronization strategy. Implementations
/// must commit atomically and roll back when a transaction is dropped.
pub trait Storage: Send + Sync {
    /// Begin a write-capable transaction. The transaction remains tied to this store.
    fn begin(&self) -> Result<Box<dyn StorageTransaction + '_>, StorageError>;

    /// Perform a local health check without making network requests.
    fn health_check(&self) -> Result<(), StorageError>;
}

/// An explicit atomic transaction over opaque namespaced byte keys.
///
/// Namespaces are stable module-owned names, not company, tenant or end-user
/// identifiers. Host applications retain responsibility for user identity
/// and authorization.
pub trait StorageTransaction {
    /// Read one value. Returns `None` when the key does not exist.
    fn get(&mut self, namespace: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;

    /// Insert or replace one value inside the current transaction.
    fn put(&mut self, namespace: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError>;

    /// Delete one value and report whether it existed.
    fn delete(&mut self, namespace: &str, key: &[u8]) -> Result<bool, StorageError>;

    /// Return entries whose keys begin with `prefix`, in bytewise key order.
    /// The limit must be between 1 and `MAX_STORAGE_SCAN_LIMIT`.
    fn scan_prefix(
        &mut self,
        namespace: &str,
        prefix: &[u8],
        limit: usize,
    ) -> Result<Vec<StorageEntry>, StorageError>;

    /// Atomically commit all changes in this transaction.
    fn commit(self: Box<Self>) -> Result<(), StorageError>;
}
