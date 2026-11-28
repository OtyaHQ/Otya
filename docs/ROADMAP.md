# Otya Engine Roadmap

This is an engine-only, evidence-based roadmap. Milestones are complete only when implementation and acceptance evidence exist. The repository does not include a programming language, compiler, language runtime or operating-system distribution.

## E0 — Product and architecture boundary

**Goal:** define exactly what is installed and who operates it.

Deliverables:
- Engine-only product definition and Otya/OEnd boundary.
- One installation, one operator; no central customer or tenant control plane.
- Embedded-library, optional local-service and CLI model.
- Customer-operated responsibility and data ownership model.
- Module, storage and provider adapter contracts.
- Engineering influence register and platform portability strategy.
- Threat model and dependency policy.

Exit gate: architecture reviewed; no mandatory Otya cloud dependency; no hidden network calls; supported initial platform documented.

## E1 — Rust workspace, build and operator CLI

**Goal:** build and run the smallest useful engine locally for one installing operator.

Deliverables:
- Rust workspace with pinned stable toolchain and declared MSRV.
- CI for formatting, Clippy, unit tests, integration tests and dependency checks.
- Linux XDG path resolution for per-operator config, data, state, cache and optional runtime paths.
- Read-only CLI paths command with human-readable and JSON output.
- Strict configuration with schema versions and no engine-level company, tenant or end-user identity fields.
- CLI for version, doctor, paths and config validation; status, logs and self-test remain future work.
- Linux x86_64 and aarch64 build targets.
- OS-neutral core contracts and isolated platform-specific code.
- Phone-first developer environment using a repository Codespaces devcontainer and remote GitHub Actions.
- Initial Android and WebAssembly compile-compatibility checks for the portable core; no APK or Worker runtime is implied.

Exit gate: clean reproducible build in CI; CLI works on clean Linux test hosts; no secrets emitted in logs; all reported support targets are actually tested.

## E2 — Secure filesystem and installation lifecycle

**Goal:** install, operate and remove the engine without a hosted Otya service.

Deliverables:
- Explicit installation roots and safe overrides, including multiple isolated installations under one OS user.
- Secure directory creation, ownership/mode checks and symlink-safe filesystem operations.
- Signed release artifacts and checksums.
- Install package with host preflight checks.
- Per-user service profile; later optional dedicated unprivileged server-service profile.
- systemd unit with tested hardening and configurable resource limits.
- Start, stop, restart, status, doctor, logs and uninstall.
- Embedded-library packaging separate from service installation.
- Interrupted-install recovery, upgrade, rollback and uninstall.

Exit gate: fresh install, interrupted-install recovery, upgrade, rollback and uninstall tested in disposable VMs; customer data preserved as documented.

## E3 — Storage and transactions

**Goal:** reliable operator-controlled persistence.

Deliverables:
- Storage and transaction traits with conformance tests.
- Optional SQLite adapter with safe WAL/backup behavior.
- Optional operator-managed PostgreSQL adapter.
- Versioned migrations and migration preflight.
- Idempotency store and transactional outbox/inbox.
- Data export and import.

Exit gate: crash and concurrency tests pass; backup and restore verified; financial/audit modules reject ephemeral storage.

## E4 — Runtime operations and reliability

**Goal:** survive ordinary process, machine and provider failures.

Deliverables:
- Job leases and worker supervision.
- Bounded retries, backoff, circuit breakers and dead-letter handling.
- Unknown-outcome tracking and reconciliation interfaces.
- Graceful shutdown and restart recovery.
- Local health/readiness probes.
- Redacted logs, metrics and trace correlation.
- Backup schedule hooks, integrity checks and restore drills.

Exit gate: failure-injection tests prove no silent data loss, duplicate unsafe side effects or unbounded retry loops.

## E5 — Security and access control

**Goal:** protect local and remote boundaries without building an identity SaaS.

Deliverables:
- OS-principal-aware local operator boundary and authenticated local IPC where required.
- Module capability grants, least privilege and default-deny operation checks.
- A narrow host-authorization context contract; the host app remains responsible for authenticating and authorizing its users.
- Secret-provider interface and credential redaction.
- Signed webhook verification and replay protection.
- Key rotation hooks and encrypted backup support.
- Dependency audit, SBOM, signing and vulnerability process.

Exit gate: threat model reviewed; local IPC and capability tests pass; host-authorization context cannot silently become a central user or tenant directory; privileged code receives security review.

## E6 — Provider adapter framework

**Goal:** make provider connections reusable, replaceable and testable.

Deliverables:
- Shared HTTP client policy, timeout, retry and rate-limit behavior.
- Adapter manifest and capability declarations.
- Sandbox/test-mode contract and deterministic test fixtures.
- Cloudflare, GitHub, Resend and Firebase adapters as separate optional packages.
- Generic SMTP, REST, webhook and S3-compatible interfaces.
- Provider health, reconciliation and operator diagnostics.

Exit gate: each adapter passes conformance tests and can be disabled or replaced without changing engine core.

## E7 — Application SDK and first applications

**Goal:** let independent developers build websites, APIs and other applications on top of Otya without turning the engine into a monolith.

Deliverables:
- Versioned application manifest, stable app ID and engine API compatibility declaration.
- Application registry, exact route contracts, request/response limits and duplicate route detection.
- Public/authenticated access policy and host-verified capability context.
- Headless HTTP host adapters and frontend-to-API integration pattern.
- Example catalog API and responsive website proving the application boundary.
- App contract tests, starter template and eventually a project scaffolder.
- Async handlers, middleware, path parameters, request schema validation and API specification generation after the initial contract is reviewed.

Exit gate: the same app contract is tested through at least two distinct host adapters; authentication is verified by the host, capabilities are not caller-spoofable, and storage/operations are durable where the app requires them.

## E8 — First business modules

**Goal:** prove that reusable infrastructure supports real domain workflows.

Prioritized modules:
- identity integration boundary (not a new mandatory identity provider);
- communications and notifications;
- organization, staff, roles and approvals;
- accounting ledger and audit;
- retail, inventory and order workflows;
- provider-neutral payment orchestration and reconciliation;
- lending and workforce modules only after domain and legal review.

Exit gate: each module has typed domain models, migrations, host-authorized operations, audit events, tests and documented limitations. Do not claim regulated service authorization merely because a module exists.

## E9 — SDKs and host-application integrations

**Goal:** let other applications embed or call the headless engine without turning Otya itself into a mobile or desktop app.

Deliverables:
- Stable Rust SDK and versioned C ABI with opaque handles, ownership, thread and error contracts.
- Optional C ABI, Kotlin/JNI or Flutter/Dart bindings only for host applications that need in-process embedding.
- A versioned HTTPS API/client contract so host websites and mobile apps can call the operator's headless Otya runtime.
- Python bindings distributed as a separate optional package.
- Optional local IPC client library for supported desktop/server hosts.
- Cross-target compile checks as compatibility evidence, not as proof of a complete app.
- Version compatibility matrix and integration examples.

No standalone Otya Android/iOS app, APK or app-store product is planned in this repository. Android target checks do not imply an APK delivery pipeline. Any future host-application SDK must remain optional and must not become a requirement for the engine's headless runtime.

**Exit gate:** host SDK lifecycle, compatibility, ownership and authorization tests pass; HTTP API authentication and versioning are stable; optional bindings do not introduce mandatory mobile UI or platform dependencies.
 
## E10 — Extension system and mature distribution

**Goal:** support independent modules without weakening the core.

Deliverables:
- Versioned extension manifests, signatures and dependency constraints.
- First-party modules as Rust crates.
- Isolated subprocess extension protocol.
- Evaluate WASI Component Model sandboxing as an optional extension ABI.
- Standard package formats only after signing, rollback and dependency-conflict behavior are proven.
- Release channels, support windows, migration guarantees and deprecation policy.
- Committed Cargo.lock and CI builds in locked dependency mode.

Exit gate: untrusted extensions cannot access undeclared host resources; incompatible upgrades fail closed; recovery and uninstall are tested; dependency versions are reproducible.

## Always-on release requirements

Every milestone preserves:
- backward compatibility or a documented migration;
- clean CI builds and tests;
- install, update, rollback and removal evidence;
- data export, backup and restore;
- signed artifacts, checksums, SBOM and dependency review;
- explicit permissions and a documented threat model;
- privacy-first observability;
- an accurate feature/support matrix.

## Non-goals

- No Otya programming language or compiler in this repository.
- No mandatory Otya-hosted compute, database, storage or control plane.
- No replacement kernel or complete OS distribution in the initial engine project.
- No engine-managed company, customer, tenant or end-user registry.
- No assumption that a provider adapter makes an external API available, legal or reliable.
- No multi-operator shared-hosting, clustering or high-availability guarantee before separately designed and tested.
