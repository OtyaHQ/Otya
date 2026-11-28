# Linux Platform Model

## 1. Otya is a userspace engine, not a new kernel

The first Otya Engine runs as an ordinary Linux userspace library or process. It does not install a replacement kernel, kernel module, hypervisor, container runtime or Linux distribution.

Linux already supplies process scheduling, virtual memory, file permissions, users/groups, signals, sockets, filesystems, namespaces, cgroups and syscall filtering. Otya builds a reliable application/engine layer using these existing interfaces. The installer must not modify the kernel or require root for ordinary embedded or per-user operation.

## 2. One operator and one installation boundary

The installation belongs to one operator. The default Linux service profile runs under the installing OS user's UID through that user's systemd user manager.

- Embedded mode: the host application links the Otya library and owns its process lifecycle.
- Per-user service mode: the operator runs one optional Otya process under their own OS UID.
- Server service mode: an operator may explicitly select one dedicated, unprivileged service account for one isolated installation. The account owns that installation's files, database, logs and socket. This is not a shared service for unrelated operators.
- A second OS user does not automatically share the first user's service, data, configuration or socket.
- A single operator can deliberately create multiple isolated installs with separate config/data roots and separate services, but they never become a tenant pool.

The engine does not create company, tenant or end-user identities. The OS security principal is the local process boundary; the operator's name or organization is not sent to Otya. Applications above the engine own their own user/account models.

A process running under the same OS UID generally has the same discretionary filesystem authority. The local engine socket is therefore an operator trust boundary, not a per-application sandbox. Do not run untrusted programs under the same OS account and assume the engine socket is private from them. Use separate OS accounts or embedded mode with host-controlled APIs when stronger separation is needed.

## 3. Use the XDG Base Directory layout

Follow the XDG Base Directory Specification for per-user paths. Resolve the environment variables at runtime and require absolute paths. Never write durable data to the runtime directory.

| Purpose | Location |
|---|---|
| Configuration | XDG_CONFIG_HOME/otya-engine/ (default ~/.config/otya-engine/) |
| Durable engine data and SQLite DB | XDG_DATA_HOME/otya-engine/ (default ~/.local/share/otya-engine/) |
| Logs and operational state | XDG_STATE_HOME/otya-engine/ (default ~/.local/state/otya-engine/) |
| Rebuildable cache | XDG_CACHE_HOME/otya-engine/ (default ~/.cache/otya-engine/) |
| Unix socket and transient locks | XDG_RUNTIME_DIR/otya-engine/ (typically /run/user/UID/otya-engine/) |

Use restrictive permissions:
- configuration and secret reference files: owner read/write only unless deliberately shared;
- data directory and database: owner-only by default;
- state/log directory: owner-only by default;
- runtime directory: 0700;
- Unix socket: 0600.

The engine must create a runtime subdirectory with owner-only permissions and verify its owner before opening a socket. Runtime files are transient; the XDG runtime directory is not suitable for database files, backups or other durable state.

The current configuration parser does not expand tilde or environment variables inside TOML strings. Until a dedicated path resolver is implemented and tested, operators must provide a real, absolute database path. Do not write a literal tilde or variable expression and assume TOML expands it.

## 4. systemd service management

The first optional daemon should be installed as a user unit, not a system-wide root service. It runs as the installing user by virtue of that user's systemd user manager; a normal user unit does not need User= or Group= switching.

A later server deployment profile may use a system service with one dedicated unprivileged service account. That is a distinct, explicit operator decision. The installer must display the selected account and every directory it will own. It must never create a shared service for unrelated operators.

The template under packaging/linux/systemd-user/ is illustrative until the daemon executable and service integration are implemented. It must not be installed or enabled automatically by the current CLI.

Expected user-service lifecycle:
- systemctl --user daemon-reload
- systemctl --user enable --now otya-engine.service
- systemctl --user status otya-engine.service
- journalctl --user -u otya-engine.service
- systemctl --user stop otya-engine.service

By default, user services are tied to the user's login/session manager. Running the service after the user logs out requires the operator to explicitly configure lingering on their own host. The installer must never enable lingering without explaining the effect and obtaining operator approval.

systemd sandbox options vary by systemd version, kernel configuration and whether a service runs under the system or user manager. Some filesystem-namespacing options available to system services are not generally available in user services. The unit must use only tested options for the supported matrix; do not claim isolation because a directive appears in a unit file.

## 5. Linux security primitives

Use the simplest effective boundary first:

1. **UID/GID and filesystem permissions:** primary isolation between different installing users.
2. **Unix-domain sockets:** local IPC without a public TCP listener. Verify path ownership and mode; do not treat same-UID local processes as mutually untrusted.
3. **systemd resource controls/cgroups:** optional CPU, memory, task and I/O bounds where the host's user manager supports them.
4. **NoNewPrivileges and capability reduction:** prevent unnecessary privilege gain where supported.
5. **AppArmor or SELinux:** optional host policy, detected and documented rather than silently assumed.
6. **seccomp:** optional syscall filtering after the required syscall set is measured and tested.
7. **Namespaces:** optional isolation for untrusted extension processes, not a prerequisite for the engine's normal embedded operation.

The engine must not attempt to implement its own kernel-level security or claim that cgroups are an authorization system. Resource controls limit consumption; they do not decide who may access customer data. Process/filesystem permissions and application authorization serve different purposes.

## 6. Filesystem and database safety

- Use absolute, operator-selected paths for durable state.
- Create directories with restrictive modes and verify ownership.
- Reject path traversal and unsafe symlink substitution in installer-managed paths.
- Do not follow arbitrary symlinks when creating sockets, config files or backup destinations.
- Keep database, logs, caches and runtime sockets in their separate XDG locations.
- Do not store credentials in TOML; use OS keyrings or explicitly configured secret stores.
- SQLite is an optional single-node durable adapter, not a distributed database.

For live SQLite backups, use the SQLite Online Backup API or a documented consistent backup procedure. A casual copy of a database file while it is active can be unsafe, especially under WAL mode or a crash. Every backup must be verified and periodically restored in an isolated directory.

## 7. Distribution strategy

Initial target:
- Linux x86_64;
- Linux aarch64;
- explicitly listed supported distributions and libc versions.

Distribution sequence:
1. CI-built, checksummed and signed tarball for controlled testing.
2. Debian package after install/upgrade/uninstall tests.
3. RPM package after the same acceptance suite.
4. Additional desktop/service targets only after the Linux lifecycle is stable.

Do not publish a generic install pipe that executes a remote script with sudo. The operator must be able to inspect the release, verify its signature/checksum, review permissions and choose whether to install the user service.

## 8. References

- XDG Base Directory Specification: https://specifications.freedesktop.org/basedir/
- Linux kernel documentation: https://docs.kernel.org/
- systemd execution and sandboxing: https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html
- systemd user unit search paths: https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html
- SQLite Online Backup API: https://sqlite.org/backup.html

These references inform design; Otya still needs its own platform tests and security review.
