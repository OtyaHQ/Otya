#![cfg(target_os = "linux")]

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use rustix::{
    fs::{Mode, OFlags, fchmod, fstat, mkdirat, open, openat},
    io::Errno,
    process::geteuid,
};
use serde::Serialize;
use thiserror::Error;

use crate::paths::EnginePaths;

/// Result of creating or validating one private Otya-owned directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PreparedDirectory {
    pub purpose: &'static str,
    pub path: PathBuf,
    pub created: bool,
    pub permissions_hardened: bool,
}

/// A filesystem path was unsafe or could not be prepared.
#[derive(Debug, Error)]
pub enum SecureFsError {
    #[error(
        "Otya per-user directories must not be provisioned as root; run as the installing unprivileged OS user"
    )]
    RunningAsRoot,
    #[error("Otya directory path must be absolute: {0}")]
    PathNotAbsolute(PathBuf),
    #[error("Otya directory path contains an unsupported component: {0}")]
    UnsupportedPathComponent(PathBuf),
    #[error("Otya directory path has no final directory component: {0}")]
    MissingFinalComponent(PathBuf),
    #[error(
        "unsafe parent directory {path}: owner uid {owner_uid}, mode {mode:o}; parent directories must be owned by root or the current operator and must not be group/world writable (except root-owned sticky directories)"
    )]
    UnsafeParent {
        path: PathBuf,
        owner_uid: u64,
        mode: u32,
    },
    #[error(
        "Otya directory {path} is owned by uid {actual_uid}, expected current effective uid {expected_uid}"
    )]
    WrongOwner {
        path: PathBuf,
        expected_uid: u64,
        actual_uid: u64,
    },
    #[error("filesystem operation {operation} failed for {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Create the operator's private directories or verify that existing ones are safe.
///
/// Each path component is opened relative to the already-open parent directory
/// with O_NOFOLLOW. This prevents an intermediate symlink from redirecting an
/// operation outside the selected path. Newly created directories use 0700;
/// an existing final Otya directory owned by this operator is changed to 0700.
///
/// This function is Linux-specific, requires an unprivileged effective UID,
/// and never deletes existing files or directories.
pub fn prepare_engine_directories(
    paths: &EnginePaths,
) -> Result<Vec<PreparedDirectory>, SecureFsError> {
    if geteuid().is_root() {
        return Err(SecureFsError::RunningAsRoot);
    }

    let mut prepared = Vec::new();
    let targets = [
        ("configuration", paths.config_dir.as_path()),
        ("data", paths.data_dir.as_path()),
        ("state", paths.state_dir.as_path()),
        ("cache", paths.cache_dir.as_path()),
    ];

    for (purpose, path) in targets {
        prepared.push(ensure_private_directory(purpose, path)?);
    }

    if let Some(runtime_dir) = &paths.runtime_dir {
        prepared.push(ensure_private_directory("runtime", runtime_dir)?);
    }

    Ok(prepared)
}

fn ensure_private_directory(
    purpose: &'static str,
    path: &Path,
) -> Result<PreparedDirectory, SecureFsError> {
    if !path.is_absolute() {
        return Err(SecureFsError::PathNotAbsolute(path.to_path_buf()));
    }

    let expected_uid = geteuid().as_raw() as u64;
    let mut current = open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| io_error("open filesystem root", path, error))?;

    let mut components = path.components().peekable();
    let mut final_created = false;
    let mut final_hardened = false;
    let mut found_final = false;

    while let Some(component) = components.next() {
        match component {
            Component::RootDir => continue,
            Component::Normal(name) => {
                let is_final = components.peek().is_none();
                let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW;

                let (next, created) = match openat(&current, name, flags, Mode::empty()) {
                    Ok(directory) => (directory, false),
                    Err(error) if error == Errno::NOENT => {
                        match mkdirat(&current, name, Mode::from_raw_mode(0o700)) {
                            Ok(()) => {}
                            Err(create_error) if create_error == Errno::EXIST => {}
                            Err(create_error) => {
                                return Err(io_error("create directory", path, create_error));
                            }
                        }

                        let directory =
                            openat(&current, name, flags, Mode::empty()).map_err(|open_error| {
                                io_error("open newly created directory", path, open_error)
                            })?;
                        (directory, true)
                    }
                    Err(error) => {
                        return Err(io_error(
                            "open directory without following symlinks",
                            path,
                            error,
                        ));
                    }
                };

                let metadata =
                    fstat(&next).map_err(|error| io_error("inspect directory", path, error))?;
                let mode = metadata.st_mode;
                let owner_uid = u64::from(metadata.st_uid);

                // DIRECTORY plus O_NOFOLLOW must yield a directory, but keep
                // an explicit type check so this invariant is visible here.
                if mode & 0o170000 != 0o040000 {
                    return Err(io_error("validate directory type", path, Errno::NOTDIR));
                }

                if is_final {
                    found_final = true;
                    if owner_uid != expected_uid {
                        return Err(SecureFsError::WrongOwner {
                            path: path.to_path_buf(),
                            expected_uid,
                            actual_uid: owner_uid,
                        });
                    }

                    if mode & 0o7777 != 0o700 {
                        fchmod(&next, Mode::from_raw_mode(0o700)).map_err(|error| {
                            io_error("set private directory permissions", path, error)
                        })?;
                        final_hardened = true;
                    }
                    final_created = created;
                } else {
                    let owned_by_trusted_principal = owner_uid == 0 || owner_uid == expected_uid;
                    let group_or_world_writable = mode & 0o022 != 0;
                    let safe_sticky_parent = owner_uid == 0 && mode & 0o1000 != 0;

                    if !owned_by_trusted_principal
                        || (group_or_world_writable && !safe_sticky_parent)
                    {
                        return Err(SecureFsError::UnsafeParent {
                            path: path.to_path_buf(),
                            owner_uid,
                            mode,
                        });
                    }
                }

                current = next;
            }
            Component::CurDir | Component::ParentDir | Component::Prefix(_) => {
                return Err(SecureFsError::UnsupportedPathComponent(path.to_path_buf()));
            }
        }
    }

    if !found_final {
        return Err(SecureFsError::MissingFinalComponent(path.to_path_buf()));
    }

    Ok(PreparedDirectory {
        purpose,
        path: path.to_path_buf(),
        created: final_created,
        permissions_hardened: final_hardened,
    })
}

fn io_error(operation: &'static str, path: &Path, error: Errno) -> SecureFsError {
    SecureFsError::Io {
        operation,
        path: path.to_path_buf(),
        source: error.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn temporary_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "otya-secure-fs-{}-{nonce}-{label}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("test root should be created");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .expect("test root should be private");
        path
    }

    #[test]
    fn creates_private_nested_directories_and_is_idempotent() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("nested");
        let target = root.join("operator").join("otya-engine");
        let first = ensure_private_directory("data", &target).expect("directory should be created");
        assert!(first.created);
        assert_eq!(
            fs::metadata(&target)
                .expect("target should exist")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );

        let second =
            ensure_private_directory("data", &target).expect("existing directory should validate");
        assert!(!second.created);
        assert!(!second.permissions_hardened);

        fs::remove_dir_all(root).expect("test tree should be removed");
    }

    #[test]
    fn hardens_an_existing_operator_owned_directory() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("mode");
        let target = root.join("otya-engine");
        fs::create_dir(&target).expect("target should be created");
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755))
            .expect("test permissions should be set");

        let report = ensure_private_directory("config", &target)
            .expect("owned directory should be hardened");
        assert!(report.permissions_hardened);
        assert_eq!(
            fs::metadata(&target)
                .expect("target should exist")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );

        fs::remove_dir_all(root).expect("test tree should be removed");
    }

    #[test]
    fn refuses_symlinked_path_components() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("symlink");
        let real = root.join("real");
        let link = root.join("redirect");
        fs::create_dir(&real).expect("real directory should be created");
        symlink(&real, &link).expect("test symlink should be created");

        let result = ensure_private_directory("data", &link.join("otya-engine"));
        assert!(result.is_err());

        fs::remove_dir_all(root).expect("test tree should be removed");
    }

    #[test]
    fn refuses_group_or_world_writable_non_sticky_parent() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("writable-parent");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o777))
            .expect("test parent permissions should be set");

        let result = ensure_private_directory("data", &root.join("otya-engine"));
        assert!(matches!(result, Err(SecureFsError::UnsafeParent { .. })));

        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .expect("test parent should be restored");
        fs::remove_dir_all(root).expect("test tree should be removed");
    }

    #[test]
    fn rejects_relative_paths() {
        if geteuid().is_root() {
            return;
        }
        let result = ensure_private_directory("data", Path::new("relative/otya-engine"));
        assert!(matches!(result, Err(SecureFsError::PathNotAbsolute(_))));
    }
}
