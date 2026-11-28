# Otya Engine Requirements Register

This is the authoritative gap and acceptance checklist. "Planned" does not mean implemented.

## Product and installation

- [ ] Rust workspace, pinned toolchain and declared MSRV.
- [ ] Embeddable library as the primary product artifact.
- [ ] Optional local daemon using local IPC by default.
- [ ] CLI for version, doctor, config, status, logs and self-test.
- [ ] Installer preflight for OS, architecture, storage, memory and permissions.
- [ ] Signed artifacts, checksums, SBOM and build provenance.
- [ ] Idempotent install, interrupted-install recovery, upgrade, rollback and uninstall.
- [ ] Unprivileged service account and OS service hardening.
- [ ] Supported Linux x86_64/aarch64 matrix before additional OS targets.

## Core contracts

- [ ] Typed configuration schema and backward-compatible migrations.
- [ ] Stable public Rust API and semantic versioning.
- [ ] Typed errors, request IDs and correlation context.
- [ ] Lifecycle hooks and graceful shutdown.
- [ ] Module registry, compatibility checks and capability grants.
- [ ] Storage, identity, secret, transport and clock abstractions.
- [ ] API compatibility and adapter conformance suites.

## Storage and data

- [ ] No mandatory Otya-hosted or central database.
- [ ] Optional SQLite adapter for local durable single-node data.
- [ ] Optional customer-managed PostgreSQL adapter for shared concurrency.
- [ ] Explicit ephemeral in-memory mode for tests only.
- [ ] Transaction and idempotency semantics tested under concurrency.
- [ ] Transactional outbox and webhook inbox.
- [ ] Versioned migrations, lock acquisition and interrupted-migration recovery.
- [ ] Encrypted/checksummed backup and restore verification.
- [ ] Data export, import, retention and deletion.
- [ ] Filesystem and optional object-storage adapters.

## Reliability

- [ ] Durable job leases, bounded retries and backoff with jitter.
- [ ] Circuit breakers and provider rate-limit handling.
- [ ] Unknown-outcome state and reconciliation before unsafe retries.
- [ ] Dead-letter inspection, replay authorization and audit.
- [ ] Startup recovery, graceful shutdown and process supervision.
- [ ] Full-disk, read-only disk, crash and power-loss test scenarios.
- [ ] Health/readiness probes and dependency diagnosis.
- [ ] Load, soak and benchmark baselines.

## Security and privacy

- [ ] Threat model and privilege boundary documented.
- [ ] Default-deny module capabilities.
- [ ] Operator/application/user/tenant roles separated.
- [ ] Secret references and secret-store adapter; no raw secrets in config/logs.
- [ ] Webhook signature, freshness, replay and size checks.
- [ ] Local IPC authentication and restrictive file/socket permissions.
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
- [ ] Per-adapter scopes, permissions, data flows and error matrix.

## Business modules

- [ ] Organization, user membership, roles and approvals.
- [ ] Communication preferences, consent, templates and delivery history.
- [ ] Append-only audit and accounting journal with integer minor units.
- [ ] Inventory, orders and retail workflows.
- [ ] Payment orchestration with unknown status and reconciliation.
- [ ] Lending and payroll only after explicit legal/domain review.
- [ ] Module-specific migrations, permissions, tests and export.
- [ ] No business module can bypass engine authorization or durable operation rules.

## Developer and operator experience

- [ ] Rust SDK documentation and examples.
- [ ] Flutter/Dart FFI wrapper with memory/lifecycle tests.
- [ ] Python bindings with packaging tests.
- [ ] Local diagnostics and safe support bundle.
- [ ] Upgrade compatibility matrix and deprecation policy.
- [ ] Admin/operator guide, emergency runbooks and recovery guides.
- [ ] Offline operation documented and tested for local features.

## Governance and release

- [ ] Approved software license and trademark policy.
- [ ] Contributor terms and code ownership.
- [ ] Release signing-key custody and rotation.
- [ ] Supported-version and security-fix policy.
- [ ] Private vulnerability reporting and coordinated disclosure.
- [ ] Release notes, known limitations and platform test evidence.

## Release rule

No item is complete because it appears in code or documentation. Each checked item requires a link to implementation and test evidence, including the exact version and supported platform. Production-critical features require failure and recovery evidence.
