#![forbid(unsafe_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use otya_engine_core::{DiagnosticSeverity, discover_engine_paths_for, parse_config};

#[cfg(target_os = "linux")]
use otya_engine_core::{DEFAULT_CONFIG_FILE_NAME, starter_config_toml};

#[derive(Debug, Parser)]
#[command(
    name = "otya",
    version,
    about = "Operate a locally installed Otya Engine",
    long_about = "Otya Engine runs in the environment selected and operated by its installer. These commands perform local checks only unless an explicit create or install action is selected."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print engine version and build information.
    Version,
    /// Inspect local host information without contacting external services.
    Doctor {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show or explicitly prepare this operator's local engine directories.
    Paths {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
        /// Explicitly create private directories and enforce owner-only permissions.
        #[arg(long)]
        create: bool,
        /// Use a separate local installation namespace (1-48 lowercase letters, digits, underscores or hyphens).
        #[arg(long, value_name = "NAME")]
        instance: Option<String>,
    },
    /// Validate operator-owned engine configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Create a private starter configuration without overwriting existing files.
    Init {
        /// Use a separate local installation namespace (1-48 lowercase letters, digits, underscores or hyphens).
        #[arg(long, value_name = "NAME")]
        instance: Option<String>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Parse and validate a TOML configuration file.
    Validate {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Version => {
            println!("Otya Engine {}", env!("CARGO_PKG_VERSION"));
            println!("mode: local / operator-controlled");
        }
        Command::Doctor { json } => {
            let report = doctor_report();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| format!("could not encode diagnostics: {error}"))?
                );
            } else {
                println!("Otya Engine local diagnostics");
                println!("version: {}", env!("CARGO_PKG_VERSION"));
                println!(
                    "host: {} / {}",
                    std::env::consts::OS,
                    std::env::consts::ARCH
                );
                println!("network: not used by this diagnostic");
                println!("result: {}", report.result);
                for check in report.checks {
                    println!("- {} [{}]: {}", check.name, check.status, check.detail);
                }
            }
        }
        Command::Paths {
            json,
            create,
            instance,
        } => show_paths(json, create, instance.as_deref())?,
        Command::Config { command } => match command {
            ConfigCommand::Init { instance, json } => init_config(instance.as_deref(), json)?,
            ConfigCommand::Validate { file, json } => validate_config(&file, json)?,
        },
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct DoctorReport {
    /// Diagnostic execution status, not a certification that the host is production-ready.
    result: &'static str,
    checks: Vec<DoctorCheck>,
}

#[derive(serde::Serialize)]
struct DoctorCheck {
    name: &'static str,
    status: &'static str,
    detail: String,
}

#[derive(serde::Serialize)]
struct PreparedDirectoryView {
    purpose: String,
    path: String,
    created: bool,
    permissions_hardened: bool,
}

fn doctor_report() -> DoctorReport {
    DoctorReport {
        result: "diagnostic_completed",
        checks: vec![
            DoctorCheck {
                name: "execution_mode",
                status: "informational",
                detail: "Local-only diagnostics; no provider or Otya service contacted.".to_owned(),
            },
            DoctorCheck {
                name: "host",
                status: "detected",
                detail: format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
            },
            DoctorCheck {
                name: "service",
                status: "not_implemented",
                detail: "The optional local service is not part of this initial CLI milestone."
                    .to_owned(),
            },
        ],
    }
}

fn show_paths(json: bool, create: bool, instance: Option<&str>) -> Result<(), String> {
    let paths = discover_engine_paths_for(instance).map_err(|error| error.to_string())?;
    let prepared = if create {
        #[cfg(target_os = "linux")]
        {
            otya_engine_core::prepare_engine_directories(&paths)
                .map_err(|error| {
                    format!(
                        "{error}. Earlier directories may already have been prepared; existing files were not deleted."
                    )
                })?
                .into_iter()
                .map(|directory| PreparedDirectoryView {
                    purpose: directory.purpose.to_owned(),
                    path: directory.path.display().to_string(),
                    created: directory.created,
                    permissions_hardened: directory.permissions_hardened,
                })
                .collect::<Vec<_>>()
        }
        #[cfg(not(target_os = "linux"))]
        {
            return Err("directory preparation is currently implemented only for Linux".to_owned());
        }
    } else {
        Vec::new()
    };

    if json {
        let output = serde_json::json!({
            "paths": &paths,
            "create_requested": create,
            "prepared_directories": &prepared,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .map_err(|error| format!("could not encode path report: {error}"))?
        );
        return Ok(());
    }

    println!("Otya Engine paths for the current operator");
    println!(
        "instance: {}",
        paths.instance.as_deref().unwrap_or("default")
    );
    println!("config:  {}", paths.config_dir.display());
    println!("data:    {}", paths.data_dir.display());
    println!("state:   {}", paths.state_dir.display());
    println!("cache:   {}", paths.cache_dir.display());
    match &paths.runtime_dir {
        Some(path) => println!("runtime: {}", path.display()),
        None => println!("runtime: unavailable (required before enabling per-user service mode)"),
    }

    for diagnostic in &paths.diagnostics {
        println!("warning [{}]: {}", diagnostic.code, diagnostic.message);
    }

    if create {
        println!();
        println!("Private directories prepared:");
        for directory in &prepared {
            println!(
                "- {}: {} (created: {}, permissions_hardened: {})",
                directory.purpose,
                directory.path,
                directory.created,
                directory.permissions_hardened
            );
        }
        println!("Existing files and data were not deleted.");
    } else {
        println!();
        println!("Read-only path inspection: no directories or files were changed.");
        println!("Use --create to explicitly prepare owner-only Otya directories.");
    }

    Ok(())
}

fn init_config(instance: Option<&str>, json: bool) -> Result<(), String> {
    let paths = discover_engine_paths_for(instance).map_err(|error| error.to_string())?;

    #[cfg(target_os = "linux")]
    {
        let directory = otya_engine_core::prepare_engine_config_directory(&paths)
            .map_err(|error| error.to_string())?;
        let starter = parse_config(starter_config_toml())
            .map_err(|error| format!("internal starter configuration is invalid: {error}"))?;
        let validation = starter.validate();
        if !validation.is_valid() {
            return Err("internal starter configuration failed validation".to_owned());
        }

        let config_path = otya_engine_core::create_private_file(
            &paths.config_dir,
            DEFAULT_CONFIG_FILE_NAME,
            starter_config_toml().as_bytes(),
        )
        .map_err(|error| match error {
            otya_engine_core::SecureFsError::FileAlreadyExists(path) => format!(
                "{} already exists; it was not changed. Validate or back it up before making any manual changes.",
                path.display()
            ),
            other => other.to_string(),
        })?;

        if json {
            let output = serde_json::json!({
                "created": true,
                "path": config_path,
                "mode": "0600",
                "config_directory": {
                    "path": directory.path,
                    "created": directory.created,
                    "permissions_hardened": directory.permissions_hardened
                },
                "instance": paths.instance,
                "validation": validation
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|error| format!("could not encode initialization result: {error}"))?
            );
        } else {
            println!("Created private Otya Engine configuration:");
            println!("file: {}", config_path.display());
            println!("mode: 0600 (owner read/write only)");
            println!(
                "instance: {}",
                paths.instance.as_deref().unwrap_or("default")
            );
            for diagnostic in &validation.diagnostics {
                let level = match diagnostic.severity {
                    DiagnosticSeverity::Warning => "warning",
                    DiagnosticSeverity::Error => "error",
                };
                println!("[{level}] {}: {}", diagnostic.code, diagnostic.message);
            }
            println!("No existing configuration or data was overwritten.");
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = paths;
        Err(
            "secure configuration initialization is currently implemented only for Linux"
                .to_owned(),
        )
    }
}

fn validate_config(file: &Path, json: bool) -> Result<(), String> {
    let source = fs::read_to_string(file)
        .map_err(|error| format!("cannot read {}: {error}", file.display()))?;

    let config = parse_config(&source).map_err(|error| format!("{}: {error}", file.display()))?;
    let report = config.validate();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("could not encode validation report: {error}"))?
        );
    } else if report.diagnostics.is_empty() {
        println!("configuration valid: {}", file.display());
    } else {
        for diagnostic in &report.diagnostics {
            let level = match diagnostic.severity {
                DiagnosticSeverity::Warning => "warning",
                DiagnosticSeverity::Error => "error",
            };
            println!("[{level}] {}: {}", diagnostic.code, diagnostic.message);
            println!("  next step: {}", diagnostic.remediation);
        }
    }

    if report.is_valid() {
        Ok(())
    } else {
        Err("configuration validation failed".to_owned())
    }
}
