use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::CONFIG_SCHEMA_VERSION;

/// Parse a configuration document without performing I/O or resolving secrets.
pub fn parse_config(source: &str) -> Result<EngineConfig, ConfigError> {
    toml::from_str(source).map_err(ConfigError::from)
}

/// Configuration for one operator-owned local engine installation.
///
/// There are deliberately no company, tenant, customer, or end-user fields here.
/// A host application owns its own identity and account model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineConfig {
    pub schema_version: u32,
    pub engine: EngineRuntimeConfig,
    pub storage: StorageConfig,
}

impl EngineConfig {
    /// Validate configuration semantics without contacting external services.
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::default();

        if self.schema_version != CONFIG_SCHEMA_VERSION {
            report.error(
                "config.unsupported_schema",
                format!(
                    "Configuration schema {} is not supported by this engine; supported schema is {}.",
                    self.schema_version, CONFIG_SCHEMA_VERSION
                ),
                "Use the documented migration tool or the configuration template for this engine version.",
            );
        }

        match &self.storage {
            StorageConfig::Memory => {
                if self.engine.mode == ExecutionMode::UserService {
                    report.error(
                        "storage.volatile_service",
                        "User-service mode requires durable storage; memory storage loses all state when the process exits.",
                        "Configure a customer-controlled persistent adapter before enabling the user service.",
                    );
                } else {
                    report.warning(
                        "storage.volatile",
                        "Memory storage loses all state when the process exits and cannot support durable workloads.",
                        "Configure a customer-controlled persistent adapter before enabling jobs, audit or business modules.",
                    );
                }
            }
            StorageConfig::Sqlite {
                path,
                busy_timeout_ms,
            } => {
                if path.as_os_str().is_empty() || path.as_path() == Path::new(":memory:") {
                    if self.engine.mode == ExecutionMode::UserService {
                        report.error(
                            "storage.sqlite_not_durable",
                            "An in-memory SQLite database is not accepted for user-service mode.",
                            "Use a file-backed database in the operator's data directory or select another supported persistent adapter.",
                        );
                    } else {
                        report.warning(
                            "storage.sqlite_ephemeral",
                            "The configured SQLite database is in-memory and will not survive process exit.",
                            "Use a file-backed database when state must persist.",
                        );
                    }
                } else if !path.is_absolute() {
                    report.error(
                        "storage.sqlite_path_not_absolute",
                        "The SQLite database path must be absolute; relative paths depend on the process working directory.",
                        "Provide an absolute path inside the installing operator's data directory.",
                    );
                }

                if !(100..=120_000).contains(busy_timeout_ms) {
                    report.error(
                        "storage.invalid_busy_timeout",
                        "SQLite busy_timeout_ms must be between 100 and 120000 milliseconds.",
                        "Set a bounded timeout appropriate for the host workload.",
                    );
                }
            }
        }

        report
    }
}

/// Local execution mode for one operator-owned installation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// The host application owns the process lifecycle.
    Embedded,
    /// One optional service running as the installing OS user.
    UserService,
}

/// Runtime settings for the locally installed engine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineRuntimeConfig {
    pub mode: ExecutionMode,
}

/// Storage selection. No cloud database or credentials are implied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case", deny_unknown_fields)]
pub enum StorageConfig {
    /// Volatile storage, useful only for tests and ephemeral work.
    Memory,
    /// Optional operator-owned, file-backed SQLite store.
    Sqlite {
        path: PathBuf,
        #[serde(default = "default_busy_timeout_ms")]
        busy_timeout_ms: u64,
    },
}

fn default_busy_timeout_ms() -> u64 {
    5_000
}

/// A validation message safe to display in operator diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub remediation: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

/// Validation results. Warnings do not invalidate the document.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub diagnostics: Vec<ConfigDiagnostic>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }

    fn error(&mut self, code: &str, message: impl Into<String>, remediation: impl Into<String>) {
        self.diagnostics.push(ConfigDiagnostic {
            severity: DiagnosticSeverity::Error,
            code: code.to_owned(),
            message: message.into(),
            remediation: remediation.into(),
        });
    }

    fn warning(&mut self, code: &str, message: impl Into<String>, remediation: impl Into<String>) {
        self.diagnostics.push(ConfigDiagnostic {
            severity: DiagnosticSeverity::Warning,
            code: code.to_owned(),
            message: message.into(),
            remediation: remediation.into(),
        });
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration syntax or shape is invalid: {0}")]
    Toml(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
schema_version = 1

[engine]
mode = "embedded"

[storage]
backend = "sqlite"
path = "/home/test/.local/share/otya-engine/engine.db"
busy_timeout_ms = 5000
"#;

    #[test]
    fn parses_valid_single_operator_configuration() {
        let config = parse_config(VALID).expect("valid config should parse");
        assert!(config.validate().is_valid());
    }

    #[test]
    fn rejects_unknown_top_level_fields() {
        let source = format!("{VALID}\nprovider_api_key = \"must-not-be-accepted\"\n");
        assert!(parse_config(&source).is_err());
    }

    #[test]
    fn rejects_company_tenant_and_user_identity_fields() {
        let source = VALID.replace(
            "[engine]",
            "company_id = \"acme\"\ntenant_id = \"tenant-1\"\nuser_id = \"alice\"\n[engine]",
        );
        assert!(parse_config(&source).is_err());
    }

    #[test]
    fn rejects_identity_fields_inside_engine_settings() {
        let source = VALID.replace(
            "[engine]",
            "[engine]\ninstallation_id = \"vendor-tracking-id\"",
        );
        assert!(parse_config(&source).is_err());
    }

    #[test]
    fn rejects_unsupported_schema() {
        let mut config = parse_config(VALID).expect("valid config should parse");
        config.schema_version = 99;
        assert!(!config.validate().is_valid());
    }

    #[test]
    fn rejects_memory_storage_for_user_service() {
        let source = r#"
schema_version = 1
[engine]
mode = "user_service"
[storage]
backend = "memory"
"#;
        let report = parse_config(source)
            .expect("config should parse")
            .validate();
        assert!(!report.is_valid());
        assert!(report.diagnostics.iter().any(|item| {
            item.code == "storage.volatile_service" && item.severity == DiagnosticSeverity::Error
        }));
    }

    #[test]
    fn rejects_in_memory_sqlite_for_user_service_mode() {
        let source = r#"
schema_version = 1
[engine]
mode = "user_service"
[storage]
backend = "sqlite"
path = ":memory:"
busy_timeout_ms = 5000
"#;
        let report = parse_config(source).expect("config should parse").validate();
        assert!(!report.is_valid());
    }

    #[test]
    fn rejects_relative_database_paths() {
        let source = VALID.replace(
            "path = \"/home/test/.local/share/otya-engine/engine.db\"",
            "path = \"./var/otya-engine.db\"",
        );
        let report = parse_config(&source)
            .expect("config should parse")
            .validate();
        assert!(!report.is_valid());
    }

    #[test]
    fn rejects_unbounded_sqlite_busy_timeout() {
        let source = VALID.replace("busy_timeout_ms = 5000", "busy_timeout_ms = 0");
        let report = parse_config(&source).expect("config should parse").validate();
        assert!(!report.is_valid());
    }
}
