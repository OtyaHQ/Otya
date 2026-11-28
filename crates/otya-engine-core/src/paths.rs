use std::path::PathBuf;

#[cfg(target_os = "linux")]
use std::env;

use serde::Serialize;
use thiserror::Error;

/// Resolved directories for one locally operated Otya installation.
///
/// This type only resolves path names. It does not create directories, change
/// permissions, follow symlinks, or claim that a runtime directory is secure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EnginePaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub runtime_dir: Option<PathBuf>,
    pub diagnostics: Vec<PathDiagnostic>,
}

/// A non-fatal issue found while interpreting the operator's environment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PathDiagnostic {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathResolutionError {
    #[error("HOME is not set to an absolute directory")]
    HomeUnavailable,
    #[error("path discovery is currently implemented only for Linux")]
    UnsupportedPlatform,
}

/// Discover XDG paths for the current Linux operator.
///
/// Relative XDG environment overrides are ignored, as required by the XDG
/// Base Directory Specification. No filesystem changes are performed.
pub fn discover_engine_paths() -> Result<EnginePaths, PathResolutionError> {
    #[cfg(target_os = "linux")]
    {
        let home = env::var_os("HOME").map(PathBuf::from);
        let config = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
        let data = env::var_os("XDG_DATA_HOME").map(PathBuf::from);
        let state = env::var_os("XDG_STATE_HOME").map(PathBuf::from);
        let cache = env::var_os("XDG_CACHE_HOME").map(PathBuf::from);
        let runtime = env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);

        resolve_from_values(home, config, data, state, cache, runtime)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(PathResolutionError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
fn resolve_from_values(
    home: Option<PathBuf>,
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    state: Option<PathBuf>,
    cache: Option<PathBuf>,
    runtime: Option<PathBuf>,
) -> Result<EnginePaths, PathResolutionError> {
    let home = home
        .filter(|path| path.is_absolute())
        .ok_or(PathResolutionError::HomeUnavailable)?;

    let mut diagnostics = Vec::new();

    let config_base = resolve_override(
        "XDG_CONFIG_HOME",
        config,
        home.join(".config"),
        &mut diagnostics,
    );
    let data_base = resolve_override(
        "XDG_DATA_HOME",
        data,
        home.join(".local/share"),
        &mut diagnostics,
    );
    let state_base = resolve_override(
        "XDG_STATE_HOME",
        state,
        home.join(".local/state"),
        &mut diagnostics,
    );
    let cache_base = resolve_override(
        "XDG_CACHE_HOME",
        cache,
        home.join(".cache"),
        &mut diagnostics,
    );

    let runtime_dir = match runtime.filter(|path| !path.as_os_str().is_empty()) {
        Some(path) if path.is_absolute() => Some(path.join("otya-engine")),
        Some(_) => {
            diagnostics.push(PathDiagnostic {
                code: "paths.invalid_xdg_runtime_home",
                message: "XDG_RUNTIME_DIR is relative and was ignored; local service mode will require a secure runtime directory.".to_owned(),
            });
            None
        }
        None => {
            diagnostics.push(PathDiagnostic {
                code: "paths.runtime_dir_unavailable",
                message: "XDG_RUNTIME_DIR is not set; embedded use can continue, but per-user service mode must not start until a secure runtime directory is available.".to_owned(),
            });
            None
        }
    };

    Ok(EnginePaths {
        config_dir: config_base.join("otya-engine"),
        data_dir: data_base.join("otya-engine"),
        state_dir: state_base.join("otya-engine"),
        cache_dir: cache_base.join("otya-engine"),
        runtime_dir,
        diagnostics,
    })
}

#[cfg(target_os = "linux")]
fn resolve_override(
    variable: &'static str,
    value: Option<PathBuf>,
    fallback: PathBuf,
    diagnostics: &mut Vec<PathDiagnostic>,
) -> PathBuf {
    match value.filter(|path| !path.as_os_str().is_empty()) {
        Some(path) if path.is_absolute() => path,
        Some(_) => {
            diagnostics.push(PathDiagnostic {
                code: "paths.invalid_xdg_override",
                message: format!(
                    "{variable} is relative and was ignored; the XDG specification requires absolute paths."
                ),
            });
            fallback
        }
        None => fallback,
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn uses_documented_xdg_defaults() {
        let paths = resolve_from_values(
            Some(PathBuf::from("/home/operator")),
            None,
            None,
            None,
            None,
            Some(PathBuf::from("/run/user/1000")),
        )
        .expect("absolute HOME should resolve");

        assert_eq!(paths.config_dir, PathBuf::from("/home/operator/.config/otya-engine"));
        assert_eq!(
            paths.data_dir,
            PathBuf::from("/home/operator/.local/share/otya-engine")
        );
        assert_eq!(
            paths.state_dir,
            PathBuf::from("/home/operator/.local/state/otya-engine")
        );
        assert_eq!(
            paths.cache_dir,
            PathBuf::from("/home/operator/.cache/otya-engine")
        );
        assert_eq!(
            paths.runtime_dir,
            Some(PathBuf::from("/run/user/1000/otya-engine"))
        );
        assert!(paths.diagnostics.is_empty());
    }

    #[test]
    fn accepts_absolute_operator_overrides() {
        let paths = resolve_from_values(
            Some(PathBuf::from("/home/operator")),
            Some(PathBuf::from("/mnt/config")),
            Some(PathBuf::from("/mnt/data")),
            Some(PathBuf::from("/mnt/state")),
            Some(PathBuf::from("/mnt/cache")),
            Some(PathBuf::from("/run/user/1000")),
        )
        .expect("absolute paths should resolve");

        assert_eq!(paths.config_dir, PathBuf::from("/mnt/config/otya-engine"));
        assert_eq!(paths.data_dir, PathBuf::from("/mnt/data/otya-engine"));
        assert_eq!(paths.state_dir, PathBuf::from("/mnt/state/otya-engine"));
        assert_eq!(paths.cache_dir, PathBuf::from("/mnt/cache/otya-engine"));
    }

    #[test]
    fn ignores_relative_xdg_overrides_and_reports_them() {
        let paths = resolve_from_values(
            Some(PathBuf::from("/home/operator")),
            Some(PathBuf::from("relative-config")),
            Some(PathBuf::from("relative-data")),
            Some(PathBuf::from("relative-state")),
            Some(PathBuf::from("relative-cache")),
            Some(PathBuf::from("relative-runtime")),
        )
        .expect("valid HOME should resolve");

        assert_eq!(
            paths.config_dir,
            PathBuf::from("/home/operator/.config/otya-engine")
        );
        assert_eq!(
            paths.data_dir,
            PathBuf::from("/home/operator/.local/share/otya-engine")
        );
        assert_eq!(
            paths.state_dir,
            PathBuf::from("/home/operator/.local/state/otya-engine")
        );
        assert_eq!(
            paths.cache_dir,
            PathBuf::from("/home/operator/.cache/otya-engine")
        );
        assert_eq!(paths.runtime_dir, None);
        assert_eq!(
            paths
                .diagnostics
                .iter()
                .filter(|item| item.code == "paths.invalid_xdg_override")
                .count(),
            4
        );
        assert!(paths
            .diagnostics
            .iter()
            .any(|item| item.code == "paths.invalid_xdg_runtime_home"));
    }

    #[test]
    fn reports_missing_runtime_directory_without_breaking_embedded_path_resolution() {
        let paths = resolve_from_values(
            Some(PathBuf::from("/home/operator")),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("embedded paths should resolve without runtime directory");

        assert!(paths.runtime_dir.is_none());
        assert!(paths
            .diagnostics
            .iter()
            .any(|item| item.code == "paths.runtime_dir_unavailable"));
    }

    #[test]
    fn rejects_missing_or_relative_home() {
        assert_eq!(
            resolve_from_values(None, None, None, None, None, None),
            Err(PathResolutionError::HomeUnavailable)
        );
        assert_eq!(
            resolve_from_values(
                Some(PathBuf::from("relative-home")),
                None,
                None,
                None,
                None,
                None
            ),
            Err(PathResolutionError::HomeUnavailable)
        );
    }
}
