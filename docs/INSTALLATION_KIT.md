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
6. **Optional adapters:** provider or storage packages installed only when the operator needs them. The Linux SQLite adapter is a separate crate and is not linked into the core; the Linux CLI includes explicit storage commands and links the adapter only on Linux.

The embedded library does not require installation of the optional service. The optional service runs under the installing OS user. It is not shared with unrelated OS users and does not require an Otya cloud account.

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

Avoid one monolithic feature bundle. Provide named features and optional crates so small consumers do not download database, cloud or provider dependencies they do not use. Keep the core free of provider SDK dependencies.

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

Current initial commands:
- `otya version`
- `otya doctor`
- `otya paths`
- `otya paths --instance NAME`
- `otya paths --create`
- `otya paths --instance NAME --create --json`
- `otya config init [--instance NAME] [--json]`
- `otya config validate --file FILE`
- `otya storage init --config FILE` (explicitly initializes the configured SQLite database)
- `otya storage check --config FILE` (checks an existing database without creating a missing file or applying migrations)

The paths command is read-only by default. An explicit `--create` action prepares owner-only directories for the selected local installation. On Linux this operation refuses root execution, rejects symlinked path components and unsafe parent directories, verifies final-directory ownership and applies mode 0700. It does not write or overwrite configuration files. Separately, `otya config init` prepares only the config directory and exclusively creates `otya-engine.toml` with mode 0600. It refuses to overwrite any existing file (including a symlink) and does not follow symlinked parent components. See [Configuration Lifecycle](CONFIGURATION_LIFECYCLE.md).

Planned commands, not yet implemented:
- install with embedded or service mode
- atomic config replacement, backups and schema migration
- service install, start, stop, restart and status
- logs
- modules list, inspect, enable and disable
- backup create, verify and restore
- migration plan, apply and status
- update check, install and rollback
- uninstall with explicit data-preservation choice

Destructive commands require a preview, explicit confirmation and a verified backup where data is involved.

The optional Linux SQLite adapter is documented in [STORAGE_SQLITE.md](STORAGE_SQLITE.md). Building it on Linux requires the host's SQLite development library and `pkg-config` (for example, `libsqlite3-dev` and `pkg-config` on Debian/Ubuntu). It is linked against the system SQLite library rather than bundling a separate copy. It implements the core storage contract and an initial v1-to-v2 schema migration, but not a general migration runner, online backup/restore or crash recovery.

## Applications built on Otya

The engine is a reusable foundation, not a bundled application. Developers can create a separate Rust app against the application SDK, or later connect another language through a stable HTTPS client contract. Websites and phone frontends remain separate from the engine and call the app's API. See [Applications on Otya](APPLICATIONS_ON_OTYA.md) and [the example catalog app](../examples/catalog-app/README.md).

The current SDK is an experimental workspace crate and has not been published. It has a manifest/route registry and host-supplied authorization context, but does not yet provide session verification, a generic HTTP server, async storage, durable jobs or a production app lifecycle.

## Phone-first development

A contributor can use a phone browser for the full development workflow. The repository includes a Codespaces devcontainer with Rust, SQLite build dependencies and Android/WebAssembly compile targets. Open `github.dev` for lightweight edits; use a Codespace for a terminal and builds; use GitHub Actions for CI and remote target compilation. See [Phone-First Development](PHONE_FIRST_DEVELOPMENT.md). The browser-only editor does not have a terminal, and Codespaces usage is subject to account quota/billing.

The Android application is a separate future distribution: a native Kotlin or Flutter host will embed the portable Rust core through a reviewed FFI package. The current portability workflow is a core compile check only; it does not generate an APK or make the CLI/Linux SQLite adapter available on Android. See [Mobile Runtime Model](MOBILE_RUNTIME_MODEL.md).

## Headless cloud and server packaging

The engine is not shipped as an APK, desktop GUI or Otya mobile app. The portable Rust core may be embedded by a host application or compiled for supported runtimes. The first Cloudflare Worker wrapper is separate from the native CLI and SQLite adapter. Full Linux workloads should use a separately tested Linux service or OCI container. Worker, container and local service modes do not share identical filesystem or process capabilities.

The Cloudflare Rust Worker is built and optionally deployed through the repository's manual GitHub Actions workflow. It requires operator-owned Cloudflare credentials stored in GitHub Actions secrets; it does not require a developer PC. Production deployment is restricted to main. See the Worker adapter guide and the phone-first development guide.

## Linux-first packaging

The first installation profile is per-user. Respect XDG configuration, data, state, cache and runtime directories. See [Linux Platform Model](LINUX_PLATFORM_MODEL.md). Do not create a global service or root-owned shared database. A second OS user gets a separate installation. One OS user may create separate named local instances using `--instance NAME`; each has a distinct config, data, state, cache and runtime directory namespace. The local service socket is private to the installing UID; it does not isolate mutually untrusted processes sharing the same UID.

The configuration validator currently requires an absolute SQLite path. Environment variables and tilde characters inside TOML strings are not expanded.

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
