#![cfg(target_os = "linux")]

use std::{
    io::{self, Write},
    os::fd::OwnedFd,
    path::{Component, Path, PathBuf},
};

use rustix::{
    fs::{AtFlags, Mode, OFlags, fchmod, fstat, fsync, mkdirat, open, openat, unlinkat},
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

/// Result of opening and hardening one existing private regular file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PreparedFile {
    pub path: PathBuf,
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
    #[error("configuration file name must be one plain file-name component: {0}")]
    InvalidFileName(String),
    #[error("configuration file already exists and was not changed: {0}")]
    FileAlreadyExists(PathBuf),
    #[error("managed file is not a regular file: {0}")]
    NotRegularFile(PathBuf),
    #[error(
        "Otya-managed file {path} is owned by uid {actual_uid}, expected current effective uid {expected_uid}"
    )]
    WrongFileOwner {
        path: PathBuf,
        expected_uid: u64,
        actual_uid: u64,
    },
    #[error(
        "writing {path} failed ({write_error}); cleanup of the incomplete file also failed ({cleanup_error})"
    )]
    FileWriteCleanup {
        path: PathBuf,
        write_error: io::Error,
        cleanup_error: io::Error,
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
    let (_directory, created, permissions_hardened) = open_verified_private_directory(path)?;
    Ok(PreparedDirectory {
        purpose,
        path: path.to_path_buf(),
        created,
        permissions_hardened,
    })
}

/// Open a directory by walking from the filesystem root with O_NOFOLLOW.
/// Every intermediate directory must have a trusted owner and safe write mode;
/// the final Otya directory is owned by this operator and hardened to 0700.
fn open_verified_private_directory(path: &Path) -> Result<(OwnedFd, bool, bool), SecureFsError> {
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
                        let created = match mkdirat(&current, name, Mode::from_raw_mode(0o700)) {
                            Ok(()) => true,
                            Err(create_error) if create_error == Errno::EXIST => false,
                            Err(create_error) => {
                                return Err(io_error("create directory", path, create_error));
                            }
                        };
                        let directory =
                            openat(&current, name, flags, Mode::empty()).map_err(|open_error| {
                                io_error(
                                    "open directory without following symlinks",
                                    path,
                                    open_error,
                                )
                            })?;
                        (directory, created)
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

    Ok((current, final_created, final_hardened))
}

/// Prepare only the configuration directory for an explicit config-init action.
pub fn prepare_engine_config_directory(
    paths: &EnginePaths,
) -> Result<PreparedDirectory, SecureFsError> {
    if geteuid().is_root() {
        return Err(SecureFsError::RunningAsRoot);
    }
    ensure_private_directory("configuration", &paths.config_dir)
}

fn validate_file_name(file_name: &str) -> Result<(), SecureFsError> {
    if file_name.is_empty()
        || file_name == "."
        || file_name == ".."
        || file_name.contains('/')
        || file_name.contains('\0')
    {
        return Err(SecureFsError::InvalidFileName(file_name.to_owned()));
    }
    Ok(())
}

/// Exclusively create a private file inside a verified operator-owned directory.
///
/// The parent directory is re-opened without following symlinks. O_EXCL means
/// an existing file (including a symlink) is never followed or overwritten.
/// The new file is mode 0600 and its contents are synced before the parent dir.
pub fn create_private_file(
    directory: &Path,
    file_name: &str,
    contents: &[u8],
) -> Result<PathBuf, SecureFsError> {
    if geteuid().is_root() {
        return Err(SecureFsError::RunningAsRoot);
    }

    validate_file_name(file_name)?;

    let (directory_fd, _, _) = open_verified_private_directory(directory)?;
    let file_path = directory.join(file_name);
    let flags = OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC | OFlags::NOFOLLOW;
    let file_fd =
        match rustix::fs::openat(&directory_fd, file_name, flags, Mode::from_raw_mode(0o600)) {
            Ok(file) => file,
            Err(error) if error == Errno::EXIST => {
                return Err(SecureFsError::FileAlreadyExists(file_path));
            }
            Err(error) => return Err(io_error("create private file", &file_path, error)),
        };

    let mut file = std::fs::File::from(file_fd);
    let write_result = (|| -> io::Result<()> {
        fchmod(&file, Mode::from_raw_mode(0o600)).map_err(io::Error::from)?;
        file.write_all(contents)?;
        file.sync_all()
    })();

    if let Err(write_error) = write_result {
        drop(file);
        return match unlinkat(&directory_fd, file_name, AtFlags::empty()) {
            Ok(()) => Err(SecureFsError::Io {
                operation: "write private file",
                path: file_path,
                source: write_error,
            }),
            Err(cleanup_error) => Err(SecureFsError::FileWriteCleanup {
                path: file_path,
                write_error,
                cleanup_error: cleanup_error.into(),
            }),
        };
    }

    drop(file);
    fsync(&directory_fd)
        .map_err(|error| io_error("sync private file's parent directory", &file_path, error))?;

    Ok(file_path)
}

/// Open an existing operator-owned regular file without following symlinks.
///
/// The parent directory is verified with the same no-follow descriptor walk as
/// directory creation. The file must be owned by the current operator and is
/// hardened to mode 0600. This function never creates or truncates a file.
pub fn prepare_private_file(
    directory: &Path,
    file_name: &str,
) -> Result<PreparedFile, SecureFsError> {
    if geteuid().is_root() {
        return Err(SecureFsError::RunningAsRoot);
    }
    validate_file_name(file_name)?;

    let (directory_fd, _, _) = open_verified_private_directory(directory)?;
    let file_path = directory.join(file_name);
    let file_fd = openat(
        &directory_fd,
        file_name,
        OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
        Mode::empty(),
    )
    .map_err(|error| {
        io_error(
            "open managed file without following symlinks",
            &file_path,
            error,
        )
    })?;

    let metadata =
        fstat(&file_fd).map_err(|error| io_error("inspect managed file", &file_path, error))?;
    if metadata.st_mode & 0o170000 != 0o100000 {
        return Err(SecureFsError::NotRegularFile(file_path));
    }

    let expected_uid = geteuid().as_raw() as u64;
    let owner_uid = u64::from(metadata.st_uid);
    if owner_uid != expected_uid {
        return Err(SecureFsError::WrongFileOwner {
            path: file_path,
            expected_uid,
            actual_uid: owner_uid,
        });
    }

    let permissions_hardened = metadata.st_mode & 0o7777 != 0o600;
    if permissions_hardened {
        fchmod(&file_fd, Mode::from_raw_mode(0o600))
            .map_err(|error| io_error("set managed file permissions", &file_path, error))?;
    }

    Ok(PreparedFile {
        path: file_path,
        permissions_hardened,
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
    fn creates_private_file_with_owner_only_permissions() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("private-file");
        let config_dir = root.join("config");
        ensure_private_directory("configuration", &config_dir)
            .expect("config directory should be prepared");

        let file = create_private_file(&config_dir, "otya-engine.toml", b"schema_version = 1\n")
            .expect("private config file should be created");
        assert_eq!(
            fs::read(&file).expect("file should be readable"),
            b"schema_version = 1\n"
        );
        assert_eq!(
            fs::metadata(&file)
                .expect("file metadata should be available")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn refuses_to_overwrite_existing_file_or_follow_existing_symlink() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("no-overwrite");
        let config_dir = root.join("config");
        ensure_private_directory("configuration", &config_dir)
            .expect("config directory should be prepared");
        let existing = config_dir.join("otya-engine.toml");
        fs::write(&existing, b"operator data").expect("existing file should be created");

        assert!(matches!(
            create_private_file(&config_dir, "otya-engine.toml", b"new content"),
            Err(SecureFsError::FileAlreadyExists(_))
        ));
        assert_eq!(
            fs::read(&existing).expect("existing file should remain"),
            b"operator data"
        );

        let target = root.join("outside-target");
        fs::write(&target, b"target data").expect("target should be created");
        let link = config_dir.join("linked.toml");
        symlink(&target, &link).expect("symlink should be created");
        assert!(matches!(
            create_private_file(&config_dir, "linked.toml", b"replacement"),
            Err(SecureFsError::FileAlreadyExists(_))
        ));
        assert_eq!(
            fs::read(&target).expect("target should remain"),
            b"target data"
        );
    }

    #[test]
    fn rejects_file_names_that_are_not_single_components() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("invalid-file-name");
        let config_dir = root.join("config");
        ensure_private_directory("configuration", &config_dir)
            .expect("config directory should be prepared");
        for invalid in [
            "",
            ".",
            "..",
            "../outside.toml",
            "sub/config.toml",
            "/tmp/file",
        ] {
            assert!(matches!(
                create_private_file(&config_dir, invalid, b"content"),
                Err(SecureFsError::InvalidFileName(_))
            ));
        }
    }

    #[test]
    fn prepares_existing_regular_file_and_hardens_its_permissions() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("prepare-file");
        let data_dir = root.join("data");
        ensure_private_directory("data", &data_dir).expect("data directory should be prepared");
        let database = data_dir.join("engine.db");
        fs::write(&database, b"placeholder").expect("placeholder should be written");
        fs::set_permissions(&database, fs::Permissions::from_mode(0o644))
            .expect("broad permissions should be set");

        let prepared =
            prepare_private_file(&data_dir, "engine.db").expect("file should be hardened");
        assert!(prepared.permissions_hardened);
        assert_eq!(
            fs::metadata(&database)
                .expect("file metadata should be available")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        fs::remove_dir_all(root).expect("test tree should be removed");
    }

    #[test]
    fn refuses_to_open_symlinks_and_non_regular_files() {
        if geteuid().is_root() {
            return;
        }

        let root = temporary_root("prepare-file-nofollow");
        let data_dir = root.join("data");
        ensure_private_directory("data", &data_dir).expect("data directory should be prepared");
        let target = root.join("outside.db");
        fs::write(&target, b"target").expect("target should be written");
        symlink(&target, data_dir.join("linked.db")).expect("symlink should be created");

        assert!(prepare_private_file(&data_dir, "linked.db").is_err());
        fs::remove_dir_all(root).expect("test tree should be removed");
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
