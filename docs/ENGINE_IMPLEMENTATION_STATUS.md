# Otya Engine Implementation Status

**Status date:** 2026-11-28

## Implemented in the engine-only branch

- Repository scope set to Otya Engine only; old language/compiler scope removed from the active branch.
- Rust workspace with core and CLI crates.
- Strict TOML configuration parser and semantic validation.
- Configuration has no company, tenant, customer or end-user identity fields; unknown identity fields are rejected.
- Local CLI commands: `otya version`, `otya doctor`, `otya paths`, and `otya config validate`.
- Linux XDG path discovery for per-operator config, data, state, cache and optional runtime directories.
- Relative XDG overrides are ignored with diagnostics; default paths and absolute overrides are covered by unit tests.
- Path inspection is read-only: it does not create directories or claim to verify owner, mode, symlink safety or available space.
- Local commands do not make network calls.
- Comprehensive engine gap audit and staged implementation plan documented.
- Obsolete .ot language example removed from the active engine branch.

## CI evidence

- Workflow: Otya Engine CI, run 84.
- Pull request: #2, tested head commit `c3ccb70a6778b4076b3d08b3ab80bcc80bb4c1cd`.
- Stable Rust quality job: **passed** — rustfmt, Clippy, workspace tests, doctor and paths CLI smoke tests, example configuration validation and release build.
- Rust 1.85 minimum-version job: **passed** — workspace check and tests.

This verifies the Rust workspace and pure XDG path-resolution behavior at that revision. It does not verify secure directory creation, owner/mode checks, symlink-safe filesystem operations, installation on physical Linux hosts, the future service daemon, data persistence, provider APIs or production readiness.

## Not yet implemented

- Secure creation and validation of configuration, data, state, cache and runtime directories.
- Explicit alternative install roots for multiple isolated installations under the same OS user.
- Atomic configuration writes, secure file opening and symlink/path traversal protection.
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

The branch is an experimental foundation, not a production-ready engine. CI passing is necessary but not sufficient for production readiness. Installation, upgrade, migration, secure filesystem behavior, backup/restore, fault-injection, security and platform acceptance tests remain outstanding.
