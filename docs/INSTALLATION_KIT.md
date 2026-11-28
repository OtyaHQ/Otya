# Otya Engine Installation Kit

## Purpose

Otya should feel like a dependable SDK/engine package that becomes part of the installing application's environment. The closest internal reference is Omy Play's use of the third-party Media Kit packages: the application includes a shared API package and selects optional platform/native components it actually needs.

Otya is not a hosted service. Installation must not create an Otya account, phone home, open a public port or provision a database without the operator's explicit choice.

## Distribution products

The same versioned core is packaged in several forms:

1. **Rust crate:** the preferred integration for Rust applications.
2. **Native library:** a stable C ABI for host applications that cannot link Rust directly.
3. **Operator CLI:** install, configure, diagnose, migrate, back up, restore and upgrade.
4. **Optional local service:** a supervised process for durable background work and local IPC.
5. **Platform wrappers:** Flutter/Dart and Python packages built only after the underlying FFI API is stable.
6. **Optional adapters:** provider or storage packages installed only when the customer needs them.

The embedded library does not require installation of the optional service. The service does not require an Otya cloud account.

## Modular package layout

Proposed Rust workspace direction:

```text
otya/
  crates/
    otya-engine-core/
    otya-engine-config/
    otya-engine-security/
    otya-engine-storage-api/
    otya-engine-operations/
    otya-engine-modules/
    otya-engine-cli/
    otya-engine-service/
    otya-engine-ffi/
    adapters/
      sqlite/
      postgres/
      filesystem/
      smtp/
      resend/
      firebase/
      github/
      cloudflare/
  bindings/
    flutter/
    python/
  packaging/
    linux/
  docs/
```

This is a proposed layout, not a statement that these crates already exist.

Avoid one monolithic feature bundle. Provide named features and optional crates so small consumers do not download database, cloud, media or provider dependencies they do not use. Keep the core free of provider SDK dependencies.

## Installation lifecycle

The installer should:

1. Detect OS, architecture, libc/runtime requirements and available disk/memory.
2. Verify the package signature and checksum before installation.
3. Display selected components and required permissions.
4. Create dedicated directories with explicit ownership and restrictive permissions.
5. Create a dedicated unprivileged service identity only when service mode is selected.
6. Write a versioned non-secret configuration template.
7. Validate storage and required dependencies.
8. Install service definitions only after operator approval.
9. Start nothing automatically unless the operator chose that behavior.
10. Run a self-test and show a clear result.
11. Record installed version and manifest locally.

The installer must be idempotent. Re-running it must not destroy or overwrite customer data or secrets.

## CLI contract

Initial commands:

- `otya version`
- `otya doctor`
- `otya config validate`
- `otya install --mode embedded|service`
- `otya service install|start|stop|restart|status`
- `otya logs`
- `otya modules list|inspect|enable|disable`
- `otya storage check`
- `otya backup create|verify|restore`
- `otya migrate plan|apply|status`
- `otya update check|install|rollback`
- `otya uninstall --preserve-data|--remove-data`

Destructive commands require a preview, explicit confirmation and a verified backup where data is involved. These commands are target behavior, not current implemented commands.

## Linux-first packaging

Initial service packages should target supported Linux x86_64 and aarch64 distributions. Consider a signed tarball first, then Debian and RPM packages after lifecycle tests. Windows and macOS installers follow once platform-specific service, secret and filesystem behavior is proven.

A systemd unit must run under a dedicated unprivileged user, use a narrow writable state directory, deny unnecessary privileges and avoid broad host filesystem access.

## Upgrade and rollback

Every release includes:
- immutable version and platform identifiers;
- cryptographic checksum and signature;
- source commit and build provenance;
- SBOM and dependency inventory;
- supported upgrade paths;
- schema compatibility range;
- release notes and known issues.

Upgrade steps are preflighted and resumable. If a migration is destructive or irreversible, require a verified backup and document the recovery path. Do not automatically run a reverse migration that could lose new customer data. Rollback restores the previous executable and only restores data when the operator explicitly chooses a verified backup.

## Acceptance tests

- Clean install on each supported distribution and architecture.
- No network traffic during a local-only install.
- No public listening socket in embedded mode.
- Service runs without root.
- Config and data directories have expected ownership and permissions.
- Repeated installation is safe.
- Interrupted installation can recover.
- Upgrade, rollback and uninstall preserve or remove data exactly as documented.
- Signature and checksum failures stop installation.
