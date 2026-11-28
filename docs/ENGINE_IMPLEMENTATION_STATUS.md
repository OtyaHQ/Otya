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

- Configuration initializer evidence: Otya Engine CI [run 153](https://github.com/OtyaHQ/Otya/actions/runs/46140624062).
- Initial SQLite adapter baseline (before final private-file hardening and added reopen/preservation tests): Otya Engine CI [run 175](https://github.com/OtyaHQ/Otya/actions/runs/46141620082), commit `39489a3dbd77157d04058796c8f07237037ace76`.
- Pull request: #2, tested implementation commit `cb3d33e29e6de51ea13ee92ef3587782d1eec2fd`.
- Stable Rust quality job: **passed** — rustfmt, Clippy, workspace tests including initial SQLite initialization/WAL/transaction/identity tests, doctor and paths CLI smoke tests, secure config initialization with mode check, generated config validation, no-overwrite/content-preservation check, example configuration validation and release build.
- Rust 1.85 minimum-version job: **passed** — workspace compile check and tests, including the SQLite foundation.

This verifies the Linux path resolver, private directory preparation and create-only configuration initializer on the GitHub-hosted Linux runner. It does not verify installation on a range of physical Linux distributions, arbitrary custom install roots, atomic config replacement, the future service daemon, data persistence, provider APIs or production readiness.

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
