# Otya Engine Architecture

**Status:** engine-only target architecture. This document defines intended boundaries; it is not a claim that every component is implemented.

## 1. Product definition

Otya Engine is a Rust-based, independently installable and embeddable engine kit. It becomes part of the customer's application or operating environment. The customer supplies and operates the host machine, storage, network, accounts, credentials, backups and deployment environment.

Otya does not operate a mandatory cloud control plane, hosted compute service or central customer database. The engine must boot and perform local work without contacting Otya infrastructure.

The engine is distinct from OEnd, OID and Otya's existing products. Any integration with those systems is optional and uses explicit versioned contracts.

## 2. Architecture at a glance

```text
Host application / customer system
          |
          +-- Embedded Rust library (primary)
          |       |
          |       +-- Core lifecycle and configuration
          |       +-- Security and capability checks
          |       +-- Module registry
          |       +-- Storage / transaction interfaces
          |       +-- Operations / events / outbox
          |       +-- Provider adapter interfaces
          |
          +-- Optional local Otya service
          |       +-- Background jobs and schedulers
          |       +-- Webhook intake when explicitly enabled
          |       +-- Local IPC API
          |       +-- Health and diagnostic endpoints
          |
          +-- Operator CLI and installation kit
                  +-- install / configure / doctor
                  +-- status / logs / backup / restore
                  +-- upgrade / rollback / uninstall
```

The embedded library, optional service and CLI must reuse the same core contracts. The local service is not required for embedded use.

## 3. Product layers

### Core Rust library

Owns configuration parsing, lifecycle state, typed errors, request context, permissions, module registration, provider-neutral operation state and stable extension interfaces. The core must avoid hidden network access and ambient filesystem access.

### Operator CLI and installer

Owns installation checks, configuration validation, local service management, migrations, backup/restore orchestration, health diagnostics and upgrade/rollback commands. Privileged installation steps must be narrow and auditable.

### Optional local service

Owns durable background execution when the customer enables it: scheduled work, leased job processing, outbox dispatch, webhook intake, reconciliation and local API/IPC. It listens on a Unix domain socket or platform-equivalent local IPC by default. A TCP listener is opt-in, authenticated and documented.

### SDK and platform bindings

The Rust API is canonical. C ABI/FFI provides a stable native boundary for Flutter/Dart and Python packages. Each wrapper is thin and maps platform-native ownership, threading and error semantics explicitly. Do not build separate business logic implementations in Dart, Python or TypeScript.

### Adapters

Adapters connect the engine to customer-selected databases, files, secret stores, identity systems, messaging providers, cloud platforms and developer tools. They are optional, least-privilege and separately versioned.

## 4. Operating modes

1. **Embedded mode:** a host app links the Rust library. The caller provides its execution lifecycle and any required storage. No daemon is started.
2. **Local service mode:** one local engine service coordinates background work for one or more trusted local apps. Local IPC is the default.
3. **Customer-hosted mode:** a customer may choose to run the same service on their own server or VPS. The customer owns the infrastructure and availability.

A single-node installation is the initial target. Multi-node clustering, distributed scheduling and multi-region replication are not first-release promises.

## 5. Storage and database boundary

The engine is not a database product and does not host or operate a database for customers.

- Storage is defined through explicit, versioned traits and transaction contracts.
- Embedded SQLite is an optional local adapter for durable single-node operation.
- PostgreSQL is an optional customer-managed adapter for deployments needing shared concurrent access.
- In-memory storage is for tests or explicitly ephemeral workloads only.
- File and object storage are separate adapters; large binary objects should not be forced into relational tables.
- Every adapter must publish transaction semantics, locking/concurrency behavior, migration rules, backup/restore behavior and supported versions.

Modules that require durability must refuse to start if configured with an ephemeral adapter. Do not silently downgrade financial or audit data to in-memory storage.

## 6. Modules and extension boundary

Built-in first-party modules are Rust crates compiled against versioned engine interfaces. They register explicit capabilities and migrations and cannot bypass core authorization or transaction boundaries.

Third-party extensions are not arbitrary native dynamic libraries loaded into the engine process. The first safe extension strategy is a separately supervised process with a versioned, authenticated local protocol. A sandboxed WebAssembly Component Model adapter may be evaluated later as an optional isolation boundary. It is a module ABI, not an Otya programming language.

Provider-specific operations live in provider adapters. Business modules own business meaning, authorization and domain invariants. The core owns common reliability and security primitives.

## 7. Provider architecture

Cloudflare, GitHub, Resend and Firebase are optional customer-configured integrations.

- GitHub is primarily a source-control and release/CI provider; it is not required for an installed instance to run.
- Cloudflare is an optional edge, storage or deployment provider; the engine is not dependent on Cloudflare Workers.
- Resend is an optional email transport; the engine can also expose SMTP or other transport adapters.
- Firebase is optional for mobile messaging or selected client-integrity integrations; it is not the engine's mandatory identity authority or database.

Every adapter declares scopes, secrets required, outbound data, webhook verification, rate limits, timeouts, retries, idempotency support, reconciliation support, health checks and error mapping. Credentials are injected at runtime from customer-controlled secret stores.

## 8. Reliability and data lifecycle

Reliable operations require:

- durable idempotency keys and request digests;
- transactional outbox and webhook inbox patterns;
- leased jobs, bounded retries, exponential backoff and jitter;
- circuit breakers and provider rate-limit awareness;
- explicit unknown outcomes and reconciliation before unsafe retries;
- append-only audit events for sensitive operations;
- crash-safe startup, graceful shutdown and job lease recovery;
- versioned migrations and interrupted-upgrade recovery;
- encrypted, checksummed backups and restore drills;
- data export and deletion paths.

Exactly-once effects across remote providers cannot be guaranteed by the engine alone. The engine uses at-least-once delivery with idempotency and reconciliation where provider capabilities permit, and exposes uncertainty rather than claiming false success.

## 9. Security model

- Run the engine under an unprivileged OS account by default.
- Deny access unless a module is explicitly granted the needed capability.
- Keep secrets out of source, logs, diagnostic bundles and plain configuration files.
- Bind local management interfaces to local IPC by default.
- Separate operator, application, tenant and end-user permissions.
- Validate and size-limit all inbound requests and webhook bodies.
- Verify signatures, timestamps and replay identifiers before accepting webhooks.
- Pin dependencies, produce an SBOM and sign release artifacts.
- Support vulnerability reporting, patch notices and a defined security support period.
- Keep telemetry disabled by default; any export requires operator opt-in.

## 10. Existing Otya ecosystem boundary

OEnd remains the backend and integrations platform for Otya's own products. OID remains the existing person-identity service. Omy Play and Opoint retain their product data and product-specific authorization.

Otya Engine must work independently. It may later expose optional adapters for OEnd/OID, but cannot assume access to Otya cloud services or make them mandatory for startup, local storage or local business operations.

## 11. Supported-platform strategy

- Initial target: supported Linux distributions on x86_64 and aarch64.
- Embedded library API: Rust first; stable C ABI for native host bindings.
- Flutter/Dart and Python wrappers follow only after the FFI contract and lifecycle tests are stable.
- Windows and macOS service installers are later targets after the Linux lifecycle is proven.
- Cloudflare Workers/Wasm execution is an optional adapter/target, not the canonical deployment.

## 12. Release acceptance

A release is not accepted because code compiles. It must pass build, unit, integration, failure-injection, compatibility, install/upgrade/rollback/uninstall, backup/restore, security, resource-limit and documentation gates. Publish the exact platforms, features, adapters and limitations tested.
