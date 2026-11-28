# Otya Engine Implementation Status

**Status date:** 2026-11-28

## Implemented in the engine-only branch

- Repository scope set to Otya Engine only; old language/compiler scope removed from the active branch.
- Rust workspace with core and CLI crates.
- Strict TOML configuration parser and semantic validation.
- Configuration has no company, tenant, customer or end-user identity fields; unknown identity fields are rejected.
- Local CLI commands: `otya version`, `otya doctor`, `otya paths`, `otya config init`, and `otya config validate`.
- Linux XDG path discovery for per-operator config, data, state, cache and optional runtime directories.
- Named local instances with validated names and separate config/data/state/cache/runtime path namespaces.
- Explicit `otya paths --create` action to prepare the selected installation directories.
- Linux filesystem preparation walks paths through directory file descriptors, refuses symlink components, rejects unsafe parent directories and foreign-owned final directories, refuses root execution, and applies owner-only 0700 permissions to final Otya directories.
- Filesystem tests cover directory create/idempotency, permission hardening, symlink rejection, unsafe writable parent rejection, relative paths, private-file mode 0600, existing-file refusal, symlink-target refusal and invalid file names.
- CLI smoke test creates an isolated named instance on a non-root Linux runner.
- Read-only `otya paths` inspection does not create or modify files.
- Linux `otya config init` prepares only the config directory and exclusively creates `otya-engine.toml` with mode 0600; it refuses existing files and does not follow symlinked parents.
- The starter config selects embedded mode with memory storage and emits a volatility warning.
- A separate Linux-only `otya-engine-storage-sqlite` crate provides an initial optional SQLite foundation: owner-only database-file creation/opening, no-follow checks, Otya application/schema metadata, exclusive first metadata transaction, WAL mode, bounded busy timeout, explicit rollback-safe transactions and SQLite integrity checks. It is not a generic storage contract or complete migration/backup system.
- Local commands do not make network calls.
- Comprehensive engine gap audit and staged implementation plan documented.
- Obsolete .ot language example removed from the active engine branch.

## CI evidence

- Configuration initializer was verified in Otya Engine CI [run 153](https://github.com/OtyaHQ/Otya/actions/runs/46140624062); it is also included in the latest full run.
- Latest full branch verification: Otya Engine CI [run 193](https://github.com/OtyaHQ/Otya/actions/runs/46141897437), commit `f560e0a0882d75bd497f64aadf413bf49bb35c3c`.
- Stable Rust quality job: **passed** — rustfmt, Clippy, all workspace tests (including SQLite initialisation, WAL, schema identity, reopen, permission hardening, foreign-database preservation and transaction rollback), CLI smoke tests, generated/example configuration validation and release build.
- Rust 1.85 minimum-version job: **passed** — workspace compile and tests for the full workspace, including the optional SQLite crate.
- The SQLite adapter uses the host system SQLite library. CI installs `libsqlite3-dev` and `pkg-config`; it does not enable the bundled SQLite feature.

This verifies the Linux path resolver, private configuration/database file handling and initial SQLite transaction foundation on a GitHub-hosted Linux runner. It does not verify a broad physical Linux distribution/SQLite-version matrix, arbitrary custom install roots, atomic config replacement, the complete storage abstraction, full schema migration lifecycle, online backup/restore, crash recovery, the future service daemon, provider APIs or production readiness.

## Not yet implemented

- Atomic configuration replacement, backup/restore and schema migrations.
- Crash/power-loss-safe temporary-file replacement for configuration; a process interruption during first creation may leave an incomplete file that must be inspected before retrying.
- Arbitrary operator-selected root paths beyond XDG overrides and validated local instance names.
- Free-space and filesystem-capacity diagnostics.
- Full SQLite storage contract, module tables, general schema migration runner, online backup/restore and crash-recovery support.
- PostgreSQL adapter.
- Per-user service and authenticated local IPC.
- Linux installer, package signatures and service hardening.
- Committed dependency lockfile and CI enforcement of locked dependency resolution.
- Secret-store abstraction and provider clients.
- Durable job queue, scheduler, outbox, inbox and webhook verification.
- Backup, restore, migrations and crash recovery.
- Business modules and host-application authorization integration.
- Stable C ABI, Flutter/Dart and Python bindings.
- Cross-platform installers, signed release artifacts, SBOM and build provenance.

## Release status

The branch is an experimental foundation, not a production-ready engine. CI passing is necessary but not sufficient for production readiness. Installation, upgrade, migration, secure configuration lifecycle, backup/restore, fault-injection, security and multi-distribution platform acceptance tests remain outstanding.
