# Otya Engine Implementation Status

**Status date:** 2026-11-28

## Implemented in the engine-only branch

- Repository scope set to Otya Engine only.
- Language specification and compiler/self-hosting strategy removed from the active branch.
- Rust workspace with core and CLI crates.
- Strict TOML configuration parser and semantic validation.
- Configuration contains execution mode and storage only; it has no company, tenant, customer or end-user identity fields.
- Initial local CLI commands: `otya version`, `otya doctor`, `otya config validate`.
- Local diagnostics do not perform network calls.
- Configuration tests authored, including rejection of unknown company/tenant/user identity fields.
- Hosted GitHub Actions workflow for formatting, Clippy, tests, CLI smoke checks, release build and Rust 1.85 MSRV.

## Not yet implemented

- Persistent SQLite storage adapter and transaction manager.
- PostgreSQL adapter.
- Per-user service and local IPC.
- Linux installer and package signatures/service hardening.
- Secret-store abstraction and provider clients.
- Durable job queue, outbox, inbox and webhook verification.
- Backup, restore, migrations and crash recovery.
- Business modules and application authorization integration.
- Flutter and Python bindings.
- Cross-platform installers and release provenance.

## Verification

The GitHub Actions workflow is the intended Rust build environment. The workspace must not be described as compiled or passing until the workflow succeeds on the current branch. Local documentation inspection does not substitute for Rust compilation.

A passing CI job is necessary but not sufficient for production readiness. Installation, upgrade, migration, backup/restore, fault-injection and security acceptance tests remain outstanding.
