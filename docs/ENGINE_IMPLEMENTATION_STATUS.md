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
- Linux platform model and engineering influence register documented.
- GitHub Actions stable quality job passed: rustfmt, Clippy, workspace tests, CLI diagnostics, example configuration validation and release build.
- GitHub Actions minimum supported Rust 1.85 job passed: workspace check and tests.

## CI evidence

- Workflow: Otya Engine CI, run 56.
- Pull request: #2, head commit `1a9a5a5f73be27d6736a84b3fcfb72c189b1661d`.
- Stable Rust quality and test job: **passed**.
- Rust 1.85 minimum-version compile/test job: **passed**.

This evidence verifies the current Rust workspace at that PR revision. It does not verify installation on physical Linux hosts, the future service daemon, provider APIs, data persistence or production readiness.

## Not yet implemented

- Persistent SQLite storage adapter and transaction manager.
- PostgreSQL adapter.
- Per-user service and local IPC.
- Linux installer and package signatures/service hardening.
- Secret-store abstraction and provider clients.
- Durable job queue, outbox, inbox and webhook verification.
- Backup, restore, migrations and crash recovery.
- Business modules and host-application authorization integration.
- Flutter and Python bindings.
- Cross-platform installers and release provenance.

## Release status

The branch is an experimental foundation, not a production-ready engine. CI passing is necessary but not sufficient for production readiness. Installation, upgrade, migration, backup/restore, fault-injection, security and platform acceptance tests remain outstanding.
