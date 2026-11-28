#![forbid(unsafe_code)]

//! Otya Engine's small, embeddable core.
//!
//! The initial core performs local configuration and path resolution only.
//! It does not open sockets, contact providers, read credentials, or create
//! a database as a side effect.

mod config;
mod paths;

pub use config::{
    ConfigDiagnostic, ConfigError, DiagnosticSeverity, EngineConfig, EngineRuntimeConfig,
    ExecutionMode, StorageConfig, ValidationReport, parse_config,
};
pub use paths::{EnginePaths, PathDiagnostic, PathResolutionError, discover_engine_paths};

/// Version of the configuration schema understood by this core release.
pub const CONFIG_SCHEMA_VERSION: u32 = 1;
