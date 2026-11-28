# Otya Engine Requirements Register

This is the authoritative gap and acceptance checklist. An item is not complete merely because it is described. See [Engine Gap Audit](ENGINE_GAP_AUDIT.md) for current implementation status and recommended order.

## Product boundary and operator model

- [ ] One installation belongs to one installing operator; an operator may be an individual or organization.
- [ ] No company registration, tenant provisioning, shared user directory or vendor installation registration in core.
- [ ] No mandatory Otya account, hosted control plane, database, compute or telemetry.
- [ ] Host application owns its users, business identities and permissions.
- [ ] Separate operators use separate isolated installations; no implicit sharing.
- [x] One operator can create separate named local instances under its own OS account, each with separate XDG paths.
- [ ] No central heartbeat, usage metering or remote enable/disable path.

## Installation and Linux lifecycle

- [x] Rust workspace with core and CLI crates, pinned stable channel and declared MSRV.
- [x] Strict versioned TOML configuration and local CLI version/doctor/config validation.
- [x] Linux XDG path-name resolution and read-only CLI paths command, with unit tests and CI smoke test.
- [x] Explicit Linux directory preparation with owner-only permissions, parent validation, no-follow path walking and unit/CI smoke tests.
- [x] Linux create-only starter configuration initialization with mode 0600, private parent directory, no-follow traversal and no-overwrite behavior (CI-verified).
- [ ] Atomic configuration replacement, explicit backup/preservation and schema-migration behavior.
- [ ] Embedded library is the primary artifact with stable public API.
- [ ] CLI doctor validates actual filesystem ownership, permissions, symlink risk, disk space and host prerequisites.
- [ ] Arbitrary custom installation-root override beyond XDG paths and named local instances.
- [ ] Optional per-user service with owner-only local IPC.
- [ ] CLI for install, status, logs, service lifecycle and self-test.
- [ ] Installer preflight for OS, architecture, storage, memory and permissions.
- [ ] Signed artifacts, checksums, SBOM and build provenance.
- [ ] Idempotent install, interrupted-install recovery, upgrade, rollback and uninstall.
- [ ] No root requirement for embedded/per-user use.
- [ ] Supported Linux x86_64/aarch64 matrix before additional OS targets.

## Core contracts

- [ ] Typed configuration schema and migrations.
- [ ] Stable Rust API and semantic versioning.
- [ ] Typed errors, request IDs and local correlation context.
- [ ] Lifecycle hooks and graceful shutdown.
- [ ] Module registry, compatibility checks and capability grants.
- [ ] Storage, secret, transport, clock and platform abstractions.
- [ ] Adapter conformance and API compatibility tests.
- [ ] No company_id, tenant_id or end-user directory in core schema.
- [ ] No mandatory network call during startup or local operation.

## Storage and data

An initial Linux SQLite foundation exists in `otya-engine-storage-sqlite` (private file opening, schema identity metadata, WAL, transactions and integrity checks). It is not yet accepted as completing the adapter requirement: the generic storage contract, module schemas, general migration lifecycle, online backup/restore and recovery testing remain open.

- [ ] No mandatory Otya-hosted or central database.
- [ ] Optional SQLite adapter for local durable single-operator data.
- [ ] Optional operator-managed PostgreSQL where concurrent app connectivity is needed.
- [ ] In-memory mode restricted to tests or explicitly ephemeral workloads.
- [ ] Transaction and idempotency semantics tested under concurrency.
- [ ] Transactional outbox and webhook inbox.
- [ ] Versioned migrations, locks and interrupted-migration recovery.
- [ ] Encrypted/checksummed backup and restore verification.
- [ ] Data export, import, retention and deletion.
- [ ] Filesystem and optional object-storage adapters.

## Runtime and reliability

- [ ] Durable job leases, bounded retries and backoff with jitter.
- [ ] Scheduler with persisted occurrence IDs and restart recovery.
- [ ] Circuit breakers and provider rate-limit handling.
- [ ] Unknown-outcome state and reconciliation before unsafe retries.
- [ ] Dead-letter inspection, replay authorization and audit.
- [ ] Startup recovery, graceful shutdown and process supervision.
- [ ] Full-disk, read-only disk, crash and power-loss tests.
- [ ] Health/readiness probes and dependency diagnosis.
- [ ] Load, soak and benchmark baselines.

## Security and privacy

- [ ] Threat model and privilege boundary documented.
- [ ] Default-deny module capabilities.
- [ ] Host application owns end-user authentication and authorization.
- [ ] Local operator control protected by OS permissions and authenticated local IPC.
- [ ] Secret references and secret-store adapter; no raw secrets in config/logs.
- [ ] Webhook signature, freshness, replay and size checks.
- [ ] Local IPC file/socket permissions tested.
- [ ] TLS and safe outbound network defaults for remote adapters.
- [ ] Dependency audit, license review, SBOM and vulnerability response.
- [ ] Release signatures and verification before install/update.
- [ ] Telemetry off by default and diagnostic export explicitly approved.

## Integrations

- [ ] Generic REST/HTTP adapter with bounded timeouts and body limits.
- [ ] Signed webhook ingress/egress.
- [ ] SMTP adapter.
- [ ] Resend optional email adapter.
- [ ] Firebase optional push/app-integrity adapter.
- [ ] GitHub optional repository/release/Actions adapter.
- [ ] Cloudflare optional Workers/R2/D1 deployment/service adapter.
- [ ] S3-compatible object-storage adapter.
- [ ] Provider sandbox fixtures and contract tests.
- [ ] Per-adapter scopes, permissions, data-flow and error matrix.

## Developer experience and portability

- [ ] Rust SDK documentation and examples.
- [ ] Versioned C ABI with ownership, threading and error contracts.
- [ ] Flutter/Dart FFI wrapper with memory/lifecycle tests.
- [ ] Python bindings as an optional separately installable package; no embedded Python requirement.
- [ ] Local diagnostics and safe support bundle.
- [ ] OS-neutral core APIs and isolated platform-specific implementations.
- [ ] Verified Linux distribution/libc/architecture matrix.
- [ ] Windows/macOS ports only after platform lifecycle and packaging tests.
- [ ] Upgrade compatibility matrix and deprecation policy.
- [ ] Operator guide, emergency runbooks and recovery guides.
- [ ] Offline operation documented and tested for local features.

## Observability, performance and governance

- [ ] Structured local logs, rotation and secret/personal-data redaction.
- [ ] Health, readiness, metrics and trace correlation with no mandatory hosted telemetry.
- [ ] Benchmarks for startup, idle memory, binary size, throughput and latency.
- [ ] Performance budgets based on named reference hardware.
- [ ] Approved software license and trademark policy.
- [ ] Contributor terms and code ownership.
- [ ] Release signing-key custody and rotation.
- [ ] Supported-version and security-fix policy.
- [ ] Private vulnerability reporting and coordinated disclosure.
- [ ] Release notes, known limitations and platform test evidence.

## Release rule

Each completed item requires implementation and test evidence for an exact version and supported platform. Production-critical features require failure and recovery evidence.
