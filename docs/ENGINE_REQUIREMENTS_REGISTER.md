# Otya Engine Requirements Register

This is the authoritative gap and acceptance checklist. An item is not complete merely because it is described.

## Product boundary and operator model

- [ ] One installation belongs to one installing operator / OS user.
- [ ] No company registration, tenant provisioning, shared user directory or vendor installation registration in core.
- [ ] No mandatory Otya account, hosted control plane, database, compute or telemetry.
- [ ] Host application owns its users, business identities and permissions.
- [ ] Multiple operators use separate OS-user installations; no implicit sharing.
- [ ] One operator can explicitly maintain separate isolated installations with distinct paths and processes.

## Installation and Linux lifecycle

- [ ] Rust workspace, pinned toolchain and declared MSRV.
- [ ] Embeddable library is the primary artifact.
- [ ] Optional per-user service with local IPC only.
- [ ] CLI for version, doctor, config, status, logs, self-test and lifecycle.
- [ ] XDG-compliant config, data, state, cache and runtime paths.
- [ ] Installer preflight for OS, architecture, storage, memory and permissions.
- [ ] Signed artifacts, checksums, SBOM and build provenance.
- [ ] Idempotent install, interrupted-install recovery, upgrade, rollback and uninstall.
- [ ] No root requirement for embedded/per-user use.
- [ ] Supported Linux x86_64/aarch64 matrix before additional OS targets.

## Core contracts

- [ ] Typed configuration schema and migrations.
- [ ] Stable Rust API and semantic versioning.
- [ ] Typed errors, request IDs and correlation context.
- [ ] Lifecycle hooks and graceful shutdown.
- [ ] Module registry, compatibility checks and capability grants.
- [ ] Storage, secret, transport and clock abstractions.
- [ ] Adapter conformance and API compatibility tests.
- [ ] No company_id, tenant_id or end-user directory in core schema.

## Storage and data

- [ ] No mandatory Otya-hosted or central database.
- [ ] Optional SQLite adapter for local durable single-operator data.
- [ ] Optional customer-managed PostgreSQL where shared app connectivity is needed.
- [ ] In-memory mode restricted to tests or explicitly ephemeral workloads.
- [ ] Transaction and idempotency semantics tested under concurrency.
- [ ] Transactional outbox and webhook inbox.
- [ ] Versioned migrations, locks and interrupted-migration recovery.
- [ ] Encrypted/checksummed backup and restore verification.
- [ ] Data export, import, retention and deletion.
- [ ] Filesystem and optional object-storage adapters.

## Reliability

- [ ] Durable job leases, bounded retries and backoff with jitter.
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

- [ ] Generic REST/HTTP adapter.
- [ ] Signed webhook ingress/egress.
- [ ] SMTP adapter.
- [ ] Resend optional email adapter.
- [ ] Firebase optional push/app-integrity adapter.
- [ ] GitHub optional repository/release/Actions adapter.
- [ ] Cloudflare optional Workers/R2/D1 deployment/service adapter.
- [ ] S3-compatible object-storage adapter.
- [ ] Provider sandbox fixtures and contract tests.
- [ ] Per-adapter scopes, permissions, data-flow and error matrix.

## Developer and operator experience

- [ ] Rust SDK documentation and examples.
- [ ] Flutter/Dart FFI wrapper with memory/lifecycle tests.
- [ ] Python bindings with packaging tests.
- [ ] Local diagnostics and safe support bundle.
- [ ] Upgrade compatibility matrix and deprecation policy.
- [ ] Operator guide, emergency runbooks and recovery guides.
- [ ] Offline operation documented and tested for local features.

## Governance and release

- [ ] Approved software license and trademark policy.
- [ ] Contributor terms and code ownership.
- [ ] Release signing-key custody and rotation.
- [ ] Supported-version and security-fix policy.
- [ ] Private vulnerability reporting and coordinated disclosure.
- [ ] Release notes, known limitations and platform test evidence.

## Release rule

Each completed item requires implementation and test evidence for an exact version and supported platform. Production-critical features require failure and recovery evidence.
