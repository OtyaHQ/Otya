#![forbid(unsafe_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use otya_engine_core::{DiagnosticSeverity, parse_config};

#[derive(Debug, Parser)]
#[command(
    name = "otya",
    version,
    about = "Operate a locally installed Otya Engine",
    long_about = "Otya Engine runs in the environment selected and operated by its installer. These commands perform local checks only."
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
    /// Validate customer-owned engine configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
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
            println!("mode: local / customer-operated");
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
                println!("host: {} / {}", std::env::consts::OS, std::env::consts::ARCH);
                println!("network: not used by this diagnostic");
                println!("result: {}", report.result);
                for check in report.checks {
                    println!("- {} [{}]: {}", check.name, check.status, check.detail);
                }
            }
        }
        Command::Config { command } => match command {
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
                detail: "The optional local service is not part of this initial CLI milestone.".to_owned(),
            },
        ],
    }
}

fn validate_config(file: &Path, json: bool) -> Result<(), String> {
    let source = fs::read_to_string(file)
        .map_err(|error| format!("cannot read {}: {error}", file.display()))?;

    let config =
        parse_config(&source).map_err(|error| format!("{}: {error}", file.display()))?;
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
