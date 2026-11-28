# Otya Engine Architecture

**Status:** engine-only target architecture. This document defines intended boundaries; it is not a claim that every component is implemented.

## 1. Product definition

Otya Engine is a Rust-based, independently installable and embeddable engine kit. Each installation belongs to one operator: an individual or an organization choosing to run its own isolated copy. It is not a hosted SaaS product and does not provide company, customer, tenant or end-user account management.

Each installation has its own configuration, storage, local credentials, logs, runtime socket and lifecycle. The default Linux service runs under the installing OS user. An operator-managed server installation may instead use a dedicated unprivileged service account, but the account operates only that operator's isolated instance. Other unrelated operators do not share it. A person or organization may deliberately operate multiple isolated copies, but those copies do not share engine state unless the operator explicitly connects them.

Otya does not operate mandatory hosted compute, a cloud control plane or a central customer database. The engine must perform local work without contacting Otya infrastructure.

The host application owns its product users, business entities, permissions and account model. Otya Engine is unaware of those identities unless a host explicitly passes a narrowly scoped authorization context for a specific operation. The core does not store tenant IDs or maintain a global user directory.

## 2. Architecture at a glance

```text
One operator / one isolated installation
          |
          +-- Host application embeds the Rust library
          |       |
          |       +-- Core lifecycle and configuration
          |       +-- Local capability enforcement
          |       +-- Module registry
          |       +-- Storage / transaction interfaces
          |       +-- Operations / outbox / audit
          |       +-- Optional provider adapters
          |
          +-- Optional local service
          |       +-- Background jobs and scheduler
          |       +-- Webhook intake only when enabled
          |       +-- Private Unix-domain socket
          |       +-- Local health and diagnostics
          |
          +-- Operator CLI and installation kit
                  +-- install / configure / doctor
                  +-- status / logs / backup / restore
                  +-- upgrade / rollback / uninstall
```

Embedded library, optional service and CLI reuse the same core contracts. The service is not required for embedded use.

## 3. Product layers

### Application SDK and applications

The application SDK is a separate layer above the portable engine core. It defines versioned application manifests, exact route declarations, request/response contracts and host-supplied authorization context. Developers build independent applications on top of these contracts; the application owns its business logic, frontend, end-user identities and product rules. See [Applications on Otya](APPLICATIONS_ON_OTYA.md). The SDK does not turn the engine into one large business application.

A frontend website or phone client calls the application's HTTPS API. The host adapter translates requests into application SDK calls. A website can be served separately or through a runtime's static-asset feature. The portable core itself remains headless and has no mandatory UI.

### Core Rust library

Owns configuration parsing, lifecycle state, typed errors, operation context, capability checks, module registration, provider-neutral operation state and stable extension interfaces. It does not create users, companies, tenants or a remote installation record. It avoids hidden network access and ambient filesystem access.

### Operator CLI and installer

Owns local installation checks, configuration validation, service management for the selected deployment profile, migrations, backup/restore orchestration, health diagnostics and upgrade/rollback commands. Privileged steps are narrow, explicit and auditable.

### Optional service

Owns durable background execution when the operator enables it: scheduled work, leased jobs, outbox dispatch, optional webhook intake, reconciliation and local API/IPC. The first service target runs under the installing OS user's account. A later server profile may use a dedicated unprivileged service account selected by the operator. Both profiles use separate owned state paths and private local sockets. There is no public TCP listener by default.

This service is not a shared host for unrelated operators. The operator remains responsible for the host application's own user and business access model.

### SDK and platform bindings

The Rust API is canonical. The application-facing `otya-engine-sdk` currently provides an initial in-process application registry and route policy contract; it is an experimental workspace crate, not a released SDK package. The C ABI and optional Python/Flutter/Dart platform bindings are separate future integration surfaces. The Rust API is canonical. A deliberately versioned C ABI provides a native boundary for Flutter/Dart and Python packages. Each wrapper is thin and defines handle ownership, buffer lifetimes, threading and error semantics. Python support is a separately installable optional SDK; CPython is not embedded in the Otya core and Python is not a runtime requirement. Do not build separate business-logic implementations in Dart, Python or TypeScript.

### Host and provider adapters

Host adapters expose application routes through a runtime such as a native Linux HTTP service or Cloudflare Worker. Provider/storage adapters connect to operator-selected databases, files, secret stores, identity systems, messaging providers, cloud platforms and developer tools. The first Worker adapter now serves a static example website and routes its API to the sample catalog application; authentication and durable storage for real applications remain incomplete. They are optional, least-privilege and separately versioned.

## 4. Operating modes

1. **Embedded mode:** the host application links the Rust library. The application owns the process lifecycle. No daemon, listener, registration or network call is started by the engine.
2. **Per-user service mode:** the installing OS user runs one local engine service for that user's trusted applications. The service has its own owner-only state paths and local socket.
3. **Operator-managed server mode:** an operator may explicitly run one installation under a dedicated unprivileged service account on their own server. This is an operational deployment choice, not a shared Otya control plane or tenant-hosting feature.

A single-operator, single-installation deployment is the first target. Multi-operator shared hosting, tenant routing, clustering and multi-region replication are explicitly out of scope.

## 5. Storage and database boundary

The engine is not a database product and does not host or operate a database for customers.

- Storage is defined through explicit, versioned traits and transaction contracts.
- Embedded SQLite is an optional local adapter for a single operator-owned installation.
- PostgreSQL is an optional operator-managed adapter for deployments where concurrent processes or remote application clients require it. It does not make Otya a multi-tenant service.
- In-memory storage is for tests or explicitly ephemeral workloads only.
- Files and object storage are separate adapters; large binary objects should not be forced into relational tables.
- Every adapter publishes transaction semantics, locking/concurrency behavior, migration rules, backup/restore behavior and supported versions.

Modules requiring durability must refuse to start with ephemeral storage. Never silently downgrade financial or audit records to memory.

## 6. Modules and extension boundary

Built-in first-party modules are Rust crates compiled against versioned engine interfaces. They register capabilities and migrations and cannot bypass core authorization or transaction boundaries.

The engine does not supply a tenant/user directory. A business module may store records relevant to the installing operator's own application, but any end-user/company model belongs to that application/domain module and remains within that operator's chosen storage.

Third-party extensions are not arbitrary native libraries loaded into the privileged engine process. The first safe extension strategy is a separate process behind a narrow authenticated local protocol. A sandboxed WebAssembly Component Model may be evaluated later as an optional module ABI; it is not an Otya language.

## 7. Provider architecture

Cloudflare, GitHub, Resend and Firebase are optional operator-configured integrations.

- GitHub is a source-control and release/CI provider; it is not needed to run an installed engine.
- Cloudflare is an optional edge/infrastructure provider; Otya is not dependent on Workers.
- Resend is optional email transport; SMTP and other providers remain possible.
- Firebase is optional for mobile push or selected app-integrity integrations; it is not the engine's mandatory database or identity authority.

Every adapter declares scopes, secrets, outbound data, webhook verification, rate limits, timeouts, retries, idempotency, reconciliation, health checks and error mapping. Credentials are injected at runtime from operator-controlled secret stores.

## 8. Reliability and data lifecycle

Reliable operations require:
- durable idempotency keys and request digests;
- transactional outbox and webhook inbox patterns;
- leased jobs, bounded retries, exponential backoff and jitter;
- circuit breakers and provider rate-limit awareness;
- explicit unknown outcomes and reconciliation before unsafe retries;
- append-only audit events for sensitive operations;
- crash-safe startup, graceful shutdown and lease recovery;
- versioned migrations and interrupted-upgrade recovery;
- encrypted, checksummed backups and restore drills;
- data export and deletion paths.

Exactly-once effects across remote providers cannot be guaranteed by the engine alone. Use at-least-once delivery with provider idempotency and reconciliation where available, and expose uncertainty rather than claiming false success.

## 9. Linux and security model

- Run as the installing OS user by default; never require root for ordinary embedded or per-user operation.
- For an operator-managed server deployment, allow one dedicated unprivileged service account only when explicitly configured. It must not become an implicit shared service for unrelated operators.
- Keep configuration, durable data, logs, cache and runtime socket in separate per-installation directories.
- Use owner-only file and socket permissions; reject symlink/path escapes when creating managed files.
- Use Linux process isolation and systemd resource/sandbox controls where available and verified.
- Do not treat namespaces, cgroups, seccomp or AppArmor as universally available. Detect platform support and document limitations.
- Do not expose local IPC over TCP by default.
- Keep secrets out of source, plain config, logs and diagnostics.
- No tenant/end-user authorization layer in engine core. The host app authorizes its own users.

## 10. Existing Otya ecosystem boundary

OEnd remains the backend and integrations platform for Otya's own products. OID remains the existing person-identity service. Omy Play and Opoint retain product data and product-specific authorization.

Otya Engine must work independently. It may later expose optional OEnd/OID adapters, but must not assume access to Otya cloud services or require Otya identities for startup or local operation.

## 11. Supported-platform strategy

- Initial target: supported Linux distributions on x86_64 and aarch64.
- Keep domain logic and core contracts OS-neutral; isolate filesystem, process, service-manager and IPC differences behind platform modules.
- Embedded API: Rust first; a versioned C ABI for native host bindings.
- Flutter/Dart and Python wrappers follow only after FFI lifecycle tests.
- Windows and macOS installation targets follow after Linux lifecycle is reliable.
- Cloudflare Workers/Wasm is an optional adapter target, not canonical deployment.

## 12. Engineering influences

Otya adopts lessons from Linux, Python, Rust/Cargo, Unix service management, SQLite and OCI without cloning those projects. The rule is to reuse standards and host capabilities where they are proven, and build Otya's own stable engine contracts and reusable modules. See [Engineering influences](ENGINEERING_INFLUENCES.md).

## 13. Release acceptance

A release requires build, unit, integration, failure-injection, compatibility, install/upgrade/rollback/uninstall, backup/restore, security, resource-limit and documentation evidence. Publish exact platforms, features, adapters and limitations tested.
