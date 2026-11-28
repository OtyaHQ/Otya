use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::CONFIG_SCHEMA_VERSION;

/// Parse a configuration document without performing I/O or resolving secrets.
pub fn parse_config(source: &str) -> Result<EngineConfig, ConfigError> {
    toml::from_str(source).map_err(ConfigError::from)
}

/// Top-level customer-owned engine configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineConfig {
    pub schema_version: u32,
    pub instance: InstanceConfig,
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

        let id = self.instance.id.trim();
        let valid_id = !id.is_empty()
            && id.len() <= 64
            && id.as_bytes()[0].is_ascii_alphanumeric()
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
            && !id.contains("..");

        if !valid_id {
            report.error(
                "config.invalid_instance_id",
                "Instance ID must be 1–64 ASCII letters, digits, dots, underscores or hyphens; it must start with a letter or digit and cannot contain '..'.",
                "Choose a stable, non-secret instance identifier.",
            );
        }

        match &self.storage {
            StorageConfig::Memory => {
                if self.instance.mode == InstanceMode::Service {
                    report.warning(
                        "storage.volatile_service",
                        "Memory storage loses all state when the process exits and is unsuitable for durable service workloads.",
                        "Configure a customer-controlled persistent adapter before enabling durable jobs, audit or business modules.",
                    );
                }
            }
            StorageConfig::Sqlite {
                path,
                busy_timeout_ms,
            } => {
                if path.as_os_str().is_empty() || path == &PathBuf::from(":memory:") {
                    if self.instance.mode == InstanceMode::Service {
                        report.error(
                            "storage.sqlite_not_durable",
                            "An in-memory SQLite database is not accepted for service mode.",
                            "Use a file-backed database in a customer-controlled directory or select another supported persistent adapter.",
                        );
                    } else {
                        report.warning(
                            "storage.sqlite_ephemeral",
                            "The configured SQLite database is in-memory and will not survive process exit.",
                            "Use a file-backed database when state must persist.",
                        );
                    }
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

/// Local installation mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceMode {
    /// The host application owns the process lifecycle.
    Embedded,
    /// The customer explicitly runs the optional local service.
    Service,
}

/// Customer-defined instance identity and mode.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceConfig {
    pub id: String,
    pub mode: InstanceMode,
}

/// Storage selection. No cloud database or credentials are implied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case", deny_unknown_fields)]
pub enum StorageConfig {
    /// Volatile storage, useful only for tests and ephemeral work.
    Memory,
    /// Optional customer-owned, file-backed SQLite store.
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

    fn warning(
        &mut self,
        code: &str,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) {
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

[instance]
id = "customer-main"
mode = "embedded"

[storage]
backend = "sqlite"
path = "./var/otya-engine.db"
busy_timeout_ms = 5000
"#;

    #[test]
    fn parses_valid_customer_configuration() {
        let config = parse_config(VALID).expect("valid config should parse");
        assert!(config.validate().is_valid());
    }

    #[test]
    fn rejects_unknown_top_level_fields() {
        let source = format!("{VALID}\nprovider_api_key = \"must-not-be-accepted\"\n");
        assert!(parse_config(&source).is_err());
    }

    #[test]
    fn rejects_unsupported_schema() {
        let mut config = parse_config(VALID).expect("valid config should parse");
        config.schema_version = 99;
        assert!(!config.validate().is_valid());
    }

    #[test]
    fn rejects_invalid_instance_identifier() {
        let mut config = parse_config(VALID).expect("valid config should parse");
        config.instance.id = "../escape".to_owned();
        assert!(!config.validate().is_valid());
    }

    #[test]
    fn warns_when_service_uses_memory_storage() {
        let source = r#"
schema_version = 1
[instance]
id = "local"
mode = "service"
[storage]
backend = "memory"
"#;
        let report = parse_config(source).expect("config should parse").validate();
        assert!(report.is_valid());
        assert!(report.diagnostics.iter().any(|item| {
            item.code == "storage.volatile_service"
                && item.severity == DiagnosticSeverity::Warning
        }));
    }

    #[test]
    fn rejects_in_memory_sqlite_for_service_mode() {
        let source = r#"
schema_version = 1
[instance]
id = "local"
mode = "service"
[storage]
backend = "sqlite"
path = ":memory:"
busy_timeout_ms = 5000
"#;
        let report = parse_config(source).expect("config should parse").validate();
        assert!(!report.is_valid());
    }

    #[test]
    fn rejects_unbounded_sqlite_busy_timeout() {
        let source = VALID.replace("busy_timeout_ms = 5000", "busy_timeout_ms = 0");
        let report = parse_config(&source).expect("config should parse").validate();
        assert!(!report.is_valid());
    }
}
