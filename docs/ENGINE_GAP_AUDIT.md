# Otya Engine Gap Audit

**Audit date:** 2026-11-28  
**Scope:** the engine-only branch and its current implementation. This document is not a claim that the planned features are complete.

## Product invariant

Otya is independently installed and operated by one operator per isolated installation. The operator may be an individual or an organization. The engine does not need to know the operator's company, create Otya accounts, register a tenant, report an installation to Otya or join unrelated installations into a shared service.

One operator may run an application with many of its own users. Those end users, organizations, permissions and business identities belong to the host application. This product rule does not mean an Otya-powered application can have only one user.

A server installation may use a dedicated unprivileged OS service account. That is a local process/filesystem boundary chosen by the operator; it is not a shared multi-customer service. Multiple installations may communicate only when the operator explicitly configures such a connection.

## Status definitions

- **Implemented and CI-verified:** working code exists and the corresponding workflow passed at the stated revision.
- **Implemented but not fully verified:** working code exists, but a required test or platform check is missing.
- **Documented only:** architecture or requirements are written, but no working implementation exists.
- **Not started:** no meaningful implementation exists.
- **Blocked by a decision:** implementation should wait until the product contract is explicitly settled.

## Current verified foundation

At Otya Engine CI run 115, commit `942041fa685ce1e00527e4d7011669ece324d892`, the branch contains:
- Rust workspace with the Otya core and CLI crates;
- strict versioned TOML configuration parsing and validation;
- rejection of unknown configuration fields, including company, tenant and user identity fields;
- local CLI commands for version, doctor, paths and configuration validation;
- Linux XDG path resolution for config, data, state, cache and optional runtime directories;
- named local instance path separation and validation;
- explicit private-directory preparation via safe rustix directory-descriptor operations;
- no-follow path walking, parent ownership/mode checks, refusal of root execution and 0700 final Otya directories;
- tests for default paths, overrides, instance separation, unsafe instance names, create/idempotency, mode hardening, symlink rejection, unsafe parent rejection and relative paths;
- stable Rust formatting, Clippy, unit tests, CLI smoke checks, example validation and release build;
- Rust 1.85 minimum-version compile and test pass.

The filesystem tests and smoke command verify the behavior on the GitHub-hosted Linux runner. They are not a substitute for a supported distribution matrix or physical-host installation tests. Directory creation does not yet create or update configuration files.

## Gap register

| Area | Current status | What is missing before production use |
|---|---|---|
| Independent installation boundary | Partially implemented; configuration, named local paths and filesystem isolation encode the rule | End-to-end tests proving separate OS users and separate installation roots cannot read each other's state |
| No Otya control plane | Documented; current CLI has no remote calls | Automated network-egress tests proving local startup and embedded operation never phone home |
| Linux directory discovery | XDG resolver and private directory preparation implemented and CI-verified | Physical distribution tests, arbitrary custom roots, free-space checks and broader install lifecycle |
| Configuration lifecycle | Strict parsing and validation implemented | Safe create-only config generation, atomic replacement, schema migration and file-level symlink/ownership tests |
| Installer and packaging | Documented only | Signed packages, preflight, safe install, uninstall, interrupted-install recovery, upgrade and rollback |
| Service runtime | Example unit only | Real daemon, signal handling, service health, resource limits, restart behavior and tested systemd integration |
| Local IPC | Documented only | Versioned protocol, peer authentication, socket permissions, request limits and client library |
| Storage | Config accepts memory/SQLite selection; no database adapter exists | SQLite implementation, migrations, transactions, locks, integrity checking and customer-managed PostgreSQL adapter |
| Operation engine | Reliability model documented only | Operation state machine, idempotency store, outbox/inbox, leases, retry scheduler, dead-letter review and reconciliation |
| Backup and restore | Documented only | Consistent backup, checksums, operator-controlled encryption keys, restore validation, migration compatibility and recovery drills |
| Secrets | Operator-owned approach documented | Secret-provider abstraction, OS keyring/vault adapters, rotation and log-redaction tests |
| Security and authorization | Initial filesystem boundary is CI-verified; broader scope rules are documented | Threat model, capability enforcement, host-authorization context, authenticated IPC and authorization tests |
| Provider adapters | Provider mapping documented | Generic HTTP/webhook foundation, sandbox fixtures, credentials, scopes, rate limits, retries, signatures and adapter conformance tests |
| Business modules | Priority list only | Versioned domain contracts, migrations, permissions, audit events, business invariants and failure tests |
| Module and extension system | Design documented | Registry, dependency/version resolution, signatures, safe disable/uninstall and isolated third-party process runtime |
| Python SDK | Design documented | Versioned FFI contract, optional Python package, supported Python matrix and lifecycle/memory tests |
| Flutter/Dart SDK | Design documented | C ABI, Dart FFI binding, native packaging and handle/lifetime/error tests |
| Cross-platform support | Linux-first strategy documented | Verified Linux distro/libc matrix first; Windows and macOS implementations only after platform abstraction tests |
| Observability | Design documented | Structured logs, rotation, health/readiness, metrics, traces, correlation and redacted support bundle |
| Performance | Release profile configured; no benchmark baseline | Startup, idle memory, binary size, throughput, latency, concurrency and soak benchmarks with named reference hardware |
| Release engineering | Basic CI exists | Cargo.lock committed and enforced with --locked, multi-architecture builds, signed artifacts, SBOM, provenance, reproducible release process, release channels and rollback evidence |
| Governance | Security and contribution guidance exists | Approved license, trademark policy, contributor terms, vulnerability reporting, support policy and release-key custody |

## Decisions to preserve

1. **Installation ownership:** the primary boundary is one operator-controlled installation. The OS user or service account is the Linux security principal. A company may be the operator, but it is not an engine tenant.
2. **Embedded versus service:** the embedded Rust library is the primary product. The service is optional and needed only for background work or IPC among applications inside the same operator boundary.
3. **Database ownership:** no Otya-hosted database. SQLite is the simplest local durable adapter; PostgreSQL is optional and operated by the customer/operator.
4. **Business identity:** engine core never becomes a customer directory, Otya login system, tenant router or company registry. Host applications authorize their own users.
5. **Extension trust:** trusted first-party modules may be compiled Rust crates. Third-party/untrusted modules must not be loaded as arbitrary native libraries into the core process.
6. **Portability order:** Linux x86_64 and aarch64 first. A platform is supported only after build, install, update, service, storage and recovery tests pass.
7. **Language support:** Rust core first; Python and Flutter/Dart are optional client bindings. No Otya language/compiler or embedded Python runtime in this repository.

## Recommended implementation order

### E1 — Local platform discovery and operator diagnostics

- Resolve per-operator config, data, state, cache and runtime paths.
- Provide the read-only paths command and local diagnostics.
- Check absolute path inputs and report invalid environment overrides.
- Keep discovery read-only; directory creation belongs to an explicit installer command.
- Add isolated tests without mutating global process environment variables.

**Acceptance:** commands produce accurate local path diagnostics on Linux, do not make network calls and never create or modify files merely by inspecting paths.

### E2 — Secure local filesystem and configuration lifecycle

- Explicit install roots and override rules, including multiple isolated installs by one operator.
- Secure directory creation with owner-only permissions.
- Safe file creation/opening that resists symlink substitution and path traversal.
- Atomic config writes, backups and schema migration.
- Installer preview, confirmation and recovery.

**Acceptance:** repeated install is safe; wrong-owner or unsafe paths fail closed; existing user data is not overwritten.

### E3 — Storage and transaction foundation

- Storage traits and conformance suite.
- SQLite adapter with documented WAL and backup semantics.
- Migration lock/state and interrupted-migration recovery.
- Data export/import and verified backup/restore.
- PostgreSQL only as a later optional customer-managed adapter.

**Acceptance:** crash, concurrency, disk-full, backup and restore tests pass.

### E4 — Optional local service and IPC

- Service lifecycle under the operator's unprivileged account.
- Owner-only Unix socket, protocol versioning and peer checks.
- Graceful shutdown, startup recovery and bounded resources.
- Status, logs and readiness diagnostics.

**Acceptance:** service is optional, works without root, does not listen publicly and cannot cross the installation boundary.

### E5 — Durable operations and reliability

- Operation IDs, idempotency, request digests and durable state transitions.
- Transactional outbox and webhook inbox.
- Worker leases, scheduler, bounded retry and dead-letter workflows.
- Unknown outcome and reconciliation model.

**Acceptance:** fault injection proves no silent data loss or unsafe duplicate external effects.

### E6 — Security, secrets and modules

- Threat model, capability model and host-authorization context.
- Secret references and operating-system secret integrations.
- Signed module manifests and safe lifecycle.
- Isolated extension protocol.

**Acceptance:** authorization, key rotation, redaction, replay, resource-limit and module-isolation tests pass.

### E7 — Provider adapters and business modules

- Generic HTTP, webhook and SMTP contracts.
- Optional Cloudflare, GitHub, Resend and Firebase adapters.
- Adapter test fixtures and sandbox capability checks.
- First-party business modules with clear domain ownership.

**Acceptance:** adapters can be replaced/disabled without changing the core; operations expose provider uncertainty and audit outcomes.

### E8 — SDKs and broad platform distribution

- Stable C ABI.
- Python and Flutter/Dart SDKs.
- Linux packages and multi-architecture artifacts.
- Windows/macOS only after platform-specific lifecycle proof.

**Acceptance:** SDK lifecycle, version compatibility, packaging and platform tests pass.

### E9 — Production release and governance

- Signed artifacts, SBOM, build provenance and vulnerability response.
- License, contributor and trademark policies.
- Performance budgets, support window and deprecation policy.
- Recovery runbooks and known-limitations matrix.

**Acceptance:** every release has reproducible build/test evidence, signed packages and a tested recovery path.

## Non-goals

- No new kernel, Linux distribution or container runtime.
- No Otya programming language, .ot source format, compiler or VM.
- No Otya-hosted compute, mandatory account, central database or control plane.
- No engine-owned company/customer/tenant/end-user directory.
- No claim of production readiness based on documents or a successful compile alone.
