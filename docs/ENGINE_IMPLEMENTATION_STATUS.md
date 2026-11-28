# Otya Engine Implementation Status

**Status date:** 2026-11-28

## Implemented in the engine-only branch

- Repository product scope updated to Otya Engine only.
- Language specification and compiler/self-hosting strategy removed from the active branch.
- Rust workspace created with core and CLI crates.
- Strict TOML configuration parser and semantic validation added.
- Initial local CLI commands: `otya version`, `otya doctor`, `otya config validate`.
- Local-only diagnostics do not perform network calls.
- Basic configuration tests authored.
- Hosted GitHub Actions workflow added for formatting, Clippy, tests, CLI smoke checks, release build and Rust 1.85 MSRV.

## Not yet implemented

- Persistent SQLite storage adapter and transaction manager.
- PostgreSQL adapter.
- Daemon/service mode and local IPC.
- Installer, package signatures and service hardening.
- Secret-store abstraction and provider clients.
- Durable job queue, outbox, inbox and webhook verification.
- Backup, restore, migrations and crash recovery.
- Business modules, identity integration and access-control implementation.
- Flutter and Python bindings.
- Cross-platform installers and release provenance.

## Verification

The GitHub Actions workflow is the Rust build environment for this project. The Rust workspace must not be described as compiled or passing until the workflow has completed successfully on this branch. Local Python/SQLite checks do not substitute for Rust compilation.

A passing CI job is necessary but not sufficient for production readiness. The installation, upgrade, data migration, backup/restore, fault-injection and security acceptance tests remain outstanding.
