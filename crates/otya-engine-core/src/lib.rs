#![forbid(unsafe_code)]

//! Otya Engine's small, embeddable core.
//!
//! The initial core performs local configuration and path resolution only.
//! It does not open sockets, contact providers, read credentials, or create
//! a database as a side effect. Filesystem preparation is an explicit,
//! operator-invoked Linux operation.

mod config;
mod paths;

#[cfg(target_os = "linux")]
mod secure_fs;

pub use config::{
    ConfigDiagnostic, ConfigError, DEFAULT_CONFIG_FILE_NAME, DiagnosticSeverity, EngineConfig,
    EngineRuntimeConfig, ExecutionMode, StorageConfig, ValidationReport, parse_config,
    starter_config_toml,
};
pub use paths::{
    EnginePaths, PathDiagnostic, PathResolutionError, discover_engine_paths,
    discover_engine_paths_for,
};

#[cfg(target_os = "linux")]
pub use secure_fs::{
    PreparedDirectory, PreparedFile, SecureFsError, create_private_file,
    prepare_engine_config_directory, prepare_engine_directories, prepare_private_file,
};

/// Version of the configuration schema understood by this core release.
pub const CONFIG_SCHEMA_VERSION: u32 = 1;
