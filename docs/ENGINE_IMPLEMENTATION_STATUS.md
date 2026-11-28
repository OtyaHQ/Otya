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
- Filesystem tests cover create/idempotency, permission hardening, symlink rejection, unsafe writable parent rejection and relative paths.
- CLI smoke test creates an isolated named instance on a non-root Linux runner.
- Read-only `otya paths` inspection does not create or modify files.
- Linux `otya config init` prepares only the config directory and exclusively creates `otya-engine.toml` with mode 0600; it refuses existing files and does not follow symlinked parents.
- The starter config selects embedded mode with memory storage and emits a volatility warning; no persistent storage adapter is claimed to exist.
- Local commands do not make network calls.
- Comprehensive engine gap audit and staged implementation plan documented.
- Obsolete .ot language example removed from the active engine branch.

## CI evidence

- Workflow: Otya Engine CI, run 115.
- Pull request: #2, tested head commit `942041fa685ce1e00527e4d7011669ece324d892`.
- Stable Rust quality job: **passed** — rustfmt, Clippy, workspace tests, doctor and paths CLI smoke tests, named-instance directory creation, example configuration validation and release build.
- Rust 1.85 minimum-version job: **passed** — workspace compile check and tests.

This verifies the earlier Linux path resolver and private directory preparation on the GitHub-hosted Linux runner. The new create-only config initializer and its overwrite/permission smoke tests are added to the current branch but are awaiting a fresh CI result. It does not verify installation on a range of physical Linux distributions, arbitrary custom install roots, atomic config replacement, the future service daemon, data persistence, provider APIs or production readiness.

## Not yet implemented

- Atomic configuration replacement, backup/restore and schema migrations.
- Crash/power-loss-safe temporary-file replacement for configuration; a process interruption during first creation may leave an incomplete file that must be inspected before retrying.
- Arbitrary operator-selected root paths beyond XDG overrides and validated local instance names.
- Free-space and filesystem-capacity diagnostics.
- Persistent SQLite storage adapter and transaction manager.
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
