#![forbid(unsafe_code)]

//! Otya Engine's small, embeddable core.
//!
//! This crate performs local configuration parsing and validation only.
//! It does not open sockets, contact providers, read credentials, or create
//! a database as a side effect.

mod config;

pub use config::{
    ConfigDiagnostic, ConfigError, DiagnosticSeverity, EngineConfig, InstanceConfig, InstanceMode,
    StorageConfig, ValidationReport, parse_config,
};

/// Version of the configuration schema understood by this core release.
pub const CONFIG_SCHEMA_VERSION: u32 = 1;
