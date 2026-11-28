# Otya Engine Requirements Register

This is the authoritative gap and acceptance checklist. An item is not complete merely because it is described. See [Engine Gap Audit](ENGINE_GAP_AUDIT.md) for current implementation status and recommended order.

## Product boundary and operator model

- [ ] One installation belongs to one installing operator; an operator may be an individual or organization.
- [ ] No global/cross-installation company registration, tenant provisioning, shared user directory or vendor installation registration.
- [ ] Each full platform installation can maintain its own private organization/workspace/project registry and identity user/group directory without sharing it with another installation.
- [ ] No mandatory Otya account, hosted control plane, database, compute or telemetry.
- [ ] Applications can authenticate through the installation-owned identity provider while retaining their own business identities, resource permissions and authorization.
- [ ] Separate operators use separate isolated installations; no implicit sharing.
- [x] One operator can create separate named local instances under its own OS account, each with separate XDG paths.
- [ ] No central heartbeat, usage metering or remote enable/disable path.

## Private identity, access and operator UI

- [ ] Per-installation identity issuer, local user/group directory and branded login portal.
- [ ] OIDC/OAuth 2.0 provider endpoints, client registration, issuer/audience/scope validation and JWKS.
- [ ] Password, passkey/WebAuthn, TOTP MFA, invitation, recovery and secure session lifecycle.
- [ ] Signing-key rotation, session revocation, service identities and machine-to-machine authentication.
- [ ] Optional external OIDC/SAML federation without requiring an external IdP for local operation.
- [ ] Shared authentication for selected internal apps while preserving application-owned authorization.
- [ ] Project-scoped roles, capability grants, default-deny policies and privilege audit.
- [ ] Authenticated local management API and browser operator console.
- [ ] Guided non-developer workflows for members, workspaces, services, monitoring, backups and deployments.
- [ ] Reusable accessible UI kit, themes, localization, app templates and schema-driven forms.
- [ ] No plaintext secrets in UI, logs, config manifests or browser clients.
- [ ] Security threat model, independent protocol conformance tests and external review before production identity release.

## Installation and Linux lifecycle

- [x] Rust workspace with core and CLI crates, pinned stable channel and declared MSRV.
- [x] Strict versioned TOML configuration and local CLI version/doctor/config validation.
- [x] Linux XDG path-name resolution and read-only CLI paths command, with unit tests and CI smoke test.
- [x] Explicit Linux directory preparation with owner-only permissions, parent validation, no-follow path walking and unit/CI smoke tests.
- [x] Linux create-only starter configuration initialization with mode 0600, private parent directory, no-follow traversal and no-overwrite behavior (CI-verified).
- [ ] Atomic configuration replacement, explicit backup/preservation and schema-migration behavior.
- [ ] Complete self-hosted platform distribution is installable, including local supervisor, management API, browser console and first-party service modules.
- [ ] Portable embedded library remains separately consumable with stable public API.
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
- [x] Backend-neutral storage and transaction contract in core, with bounded namespaces, keys, values and prefix scans; Linux SQLite adapter and CLI init/check smoke tests passed CI [run 265](https://github.com/OtyaHQ/Otya/actions/runs/46143575958).
- [ ] Secret, transport, clock and full platform abstractions.
- [ ] Adapter conformance and API compatibility tests.
- [ ] No company_id, tenant_id or end-user directory in core schema.
- [ ] No mandatory network call during startup or local operation.

## Storage and data

The Linux SQLite adapter implements the core storage contract, namespaced key/value operations and an initial transactional schema v1-to-v2 upgrade; the adapter and CLI smoke tests passed Otya Engine CI run 265 on a GitHub-hosted Linux runner. Module-owned schemas, a general migration runner, multi-process stress, online backup/restore and recovery testing remain open.

- [ ] No mandatory Otya-hosted or central database.
- [ ] SQLite adapter production acceptance after multi-process concurrency, crash recovery, backup/restore and broader platform gates pass.
- [ ] Optional operator-managed PostgreSQL where concurrent app connectivity is needed.
- [ ] In-memory mode restricted to tests or explicitly ephemeral workloads.
- [ ] Transaction and idempotency semantics tested under concurrency.
- [ ] Transactional outbox and webhook inbox.
- [ ] Versioned migrations, locks and interrupted-migration recovery.
- [ ] Encrypted/checksummed backup and restore verification.
- [ ] Data export, import, retention and deletion.
- [ ] Filesystem and optional object-storage adapters.

## Headless runtime and remote operation

- [x] Engine product is headless-first; the phone is a browser/client control surface, not an engine installation target.
- [x] Separate Cloudflare Rust/WASM Worker host adapter and manual phone-triggered deployment workflow are committed; build/deployment verification is pending.
- [ ] Generic authenticated HTTP management/API server and stable error/versioning contract.
- [ ] Async host contracts for D1/remote storage, queues and durable operations; the current SQLite storage API is synchronous and native-first.
- [ ] Cloudflare Worker, Container and generic OCI runtime integration, lifecycle and recovery acceptance.
- [ ] Provider-neutral deployment profiles so GitHub/Cloudflare remain optional, operator-owned tooling.

## Application SDK and developer experience

- [x] Versioned application API contract and static application manifest.
- [x] Exact-path route registry with duplicate app/route rejection and engine API compatibility check.
- [x] Public/authenticated route declarations, host-supplied verified caller context and declared capability checks.
- [x] Request/response bounds, safe canonical path validation and standard JSON error responses.
- [x] Example stateless catalog application with route-level tests.
- [x] Responsive static website example calling the application API through the Cloudflare Worker adapter.
- [ ] Production-ready installation-owned identity provider, OIDC/OAuth client and host authorization adapter; caller context is only trustworthy when constructed by a verified host.
- [ ] Async application handlers and request-body forwarding, middleware, route parameters and streaming APIs.
- [ ] Language-neutral authenticated HTTPS/JSON API with published OpenAPI descriptions and a second independent host adapter.
- [ ] External developers can use released contracts from TypeScript, Python, Go, Java, .NET, PHP, Ruby and other languages without an Otya-hosted account.
- [ ] SDK versioning and generated client conformance tests.
- [ ] Application scaffolder, starter templates, package signatures, migrations and upgrade/rollback lifecycle.
- [ ] Durable storage and operations contracts for apps that need persistent state or background work.

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
- [ ] Applications may use installation-owned authentication while retaining product-specific resource authorization and business permissions.
- [ ] Local operator control protected by OS permissions and authenticated local IPC.
- [ ] Local credential vault and provider-specific secret-store adapters; no raw secrets in config/logs or frontend builds.
- [ ] Webhook signature, freshness, replay and size checks.
- [ ] Local IPC file/socket permissions tested.
- [ ] TLS and safe outbound network defaults for remote adapters.
- [ ] Dependency audit, license review, SBOM and vulnerability response.
- [ ] Release signatures and verification before install/update.
- [ ] Telemetry off by default and diagnostic export explicitly approved.

## Bring-your-own-infrastructure and provider connections

- [ ] Otya does not create provider accounts or provision paid cloud resources during setup.
- [ ] Provider catalogue distinguishes Otya-local modules from user-owned external services.
- [ ] Provider connection, resource binding, project grant and webhook endpoint have separate versioned records.
- [ ] Guided OAuth/API-token/mTLS/custom REST/OpenAPI connection workflows.
- [ ] Provider-specific least-privilege scope validation and account/resource selection.
- [ ] Local credential vault backed by OS secret store, operator-managed vault/KMS or encrypted local store.
- [ ] Saved secret values are write-only, encrypted at rest, never returned to browser and redacted from logs.
- [ ] Deployment credential is separate from application runtime secret; remote secrets are injected into provider-native secret stores.
- [ ] Development, staging and production connections/secrets/resources are isolated.
- [ ] Cloudflare adapter supports token scope checks, inventory, deployment plan, bindings and Worker secrets.
- [ ] Firebase adapter distinguishes public Firebase API configuration from privileged service-account credentials.
- [ ] Supabase adapter distinguishes publishable from secret keys and provides an RLS review checklist.
- [ ] Generic REST/OpenAPI and signed webhook connectors with bounded requests and schema validation.
- [ ] Webhooks verify signatures over raw body, timestamps, replay, event IDs, size limits and idempotency.
- [ ] Mutating, destructive, billable and production operations require a preview plan and explicit approval.
- [ ] Provider health, expiry, missing scope, quotas, status, last verified time and billing owner are visible.
- [ ] Local-only operation works with no external provider connection.
- [ ] Deployed workloads continue while local Otya Console/control plane is offline.

## Brand, white-label and UI identity

- [ ] Master brand, original logo and wordmark approved; icon-only, monochrome, reversed and favicon assets.
- [ ] Product family uses consistent Otya, Engine, Console, Identity, Connect, Studio naming.
- [ ] Light/dark design tokens and responsive UI components meet accessibility requirements.
- [ ] Installation-level organization logo, name, login portal and theme customization.
- [ ] External provider attribution, billing responsibility and data location are visible.
- [ ] Brand voice, accessibility, localization, icon and UI contribution rules documented.

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

- [x] Codespaces devcontainer for phone-accessible remote Rust development, plus phone-first workflow documentation (first launch still needs manual confirmation).
- [x] Android and WebAssembly portable-core compile workflow passed; this is compile-compatibility evidence only and does not imply an APK or mobile app.
- [x] Headless-first product boundary: Otya itself does not ship an Android/iOS app, APK or desktop GUI.
- [ ] Stable versioned HTTPS API, authentication and host-application client contract.
- [ ] Optional C ABI/JNI/Flutter/Dart host-application SDKs only if embedding is required; no standalone Otya APK pipeline.
- [ ] Cloudflare Worker runtime tests, first deployed preview, async storage/operation contract and adapter conformance.
- [ ] Cloudflare Container and generic OCI runtime profiles with durable external storage and recovery tests.
- [ ] iOS FFI, Keychain, background-task and App Store release pipeline.
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
