# Headless and Portable Runtime Model

**Product decision:** the Rust Engine core is headless and portable, while the complete self-hosted distribution includes a browser-based operator console, installation-owned login portal and reusable UI kit. It is not a hosted SaaS product, graphical operating system or mandatory Android/iOS app. Interfaces include Rust APIs, CLI, authenticated management APIs, UI clients, webhooks and provider/runtime adapters.

A phone is a supported control device. It can edit code, start a remote build, approve a deployment, monitor health and call a host application's HTTPS API. The engine itself runs on the selected server or cloud runtime and continues running independently of the phone.

## 1. One core, explicit host adapters

The architecture is split into:

1. **Portable core:** domain-neutral Rust contracts, configuration parsing, module contracts, policy decisions and deterministic business logic that do not require a particular OS or cloud provider.
2. **Host/runtime adapter:** translates a platform's request, event, storage, secrets, clock and lifecycle APIs into the core's contracts.
3. **Provider/storage adapters:** optional integrations that operators select and configure for their own accounts.
4. **Operator UI and identity services:** the installed platform provides a management console and may provide its own login portal/user directory. Each is scoped to that installation.
5. **Host application:** owns its product UI and business authorization. It may use the installation's identity issuer for sign-in, then applies its own resource permissions. A phone browser or mobile app calls that host application's API.

Do not make the portable core depend on Linux paths, a database server, a Cloudflare binding, a GitHub account, an Android activity or a graphical UI. Platform-specific capabilities are injected through adapters.

## 2. Runtime profiles

| Profile | Runtime | Suitable workloads | Important boundary |
|---|---|---|---|
| Embedded Rust library | Host process on Linux or another verified native target | Applications that want Otya code in their own process | No daemon or network listener is required |
| Linux headless service | Native unprivileged process, optional systemd supervision | Long-running jobs, local IPC, native filesystem and SQLite | The operator owns the host, UID, disk, backups and service lifecycle |
| Cloudflare Worker | Rust compiled to wasm32-unknown-unknown using workers-rs | HTTP APIs, webhooks, edge request handling and supported Worker events | Not a general Linux process: do not assume native SQLite, arbitrary filesystem access, fork/exec or a long-running daemon |
| Cloudflare Container | Linux container behind a Worker and Durable Object | Existing Linux binaries, system packages, full userspace and container-oriented workloads | Cloudflare Container images currently target linux/amd64; container disk is ephemeral by default, so durable state belongs in external storage |
| Generic OCI container | Docker/Podman-compatible Linux host or cloud container service | Full Linux userspace on a VPS, VM, container host or orchestrator | Persist data in mounted operator-managed volumes or external databases; do not assume container-local disk survives replacement |
| Other serverless runtimes | Provider-specific adapter built around the same contracts | Stateless request handlers and provider-supported background events | Add only after the target runtime's APIs, limits, retries and storage semantics are tested |

The first Cloudflare Rust Worker project lives in platforms/cloudflare-worker/. It now exposes /health and /version, serves a responsive example website from Workers Static Assets, and routes /api/v1/catalog/* to the example catalog application through the application SDK. The Worker is compile-checked separately from the main workspace. Its sample API is public, read-only and backed by static fixtures only; no production authentication or durable storage is wired yet.

The Cloudflare Worker crate is isolated in its own Cargo workspace because workers-rs 0.8.7 requires Rust 1.91. The portable engine core retains its Rust 1.85 minimum. This avoids making every native/embedded Otya consumer install a newer toolchain or depend on Cloudflare libraries.

## 3. Cloudflare service mapping

Cloudflare is an optional host. It is not the Otya control plane and does not become a central shared database for unrelated Otya installations.

Use the service that matches the data and consistency requirement:

| Requirement | Cloudflare option | Otya integration rule |
|---|---|---|
| Relational application records | D1 | Implement as a separate remote/async SQL adapter; do not pretend it is a local SQLite file |
| Per-resource coordination and strongly consistent transactional state | Durable Objects with SQLite storage | Use for explicit coordination boundaries such as one operation aggregate or one resource lock |
| Read-heavy configuration and cache data | Workers KV | Treat it as eventually consistent key/value storage, not as a transaction ledger |
| Files, media and larger binary objects | R2 | Store object data separately from transactional metadata |
| Deferred background messages | Queues | Design consumers for at-least-once delivery, duplicate messages, retries and idempotency |
| Durable multi-step work | Workflows | Use for steps that must persist, wait, retry and resume |
| Existing PostgreSQL or MySQL | Hyperdrive | Connect to the operator's existing database rather than migrating data unnecessarily |

Do not force all these services into one generic "database" abstraction. Their consistency, retry, delivery and transaction guarantees differ.

### The asynchronous storage gap

The current core Storage contract is synchronous and the SQLite adapter implements it for native Linux use. Cloudflare D1, KV, R2, Queues and Durable Objects expose asynchronous runtime operations. The current synchronous SQLite contract is not a drop-in adapter interface for those services.

Before implementing D1 or remote storage, add a separately designed asynchronous host contract and an adapter conformance suite. Keep local SQLite as an optional native adapter. Do not block a Worker event loop, emulate native filesystem access, or claim a remote binding is transaction-compatible with SQLite unless the semantics are proved.

## 4. Persistence and process lifecycle

- Native Linux mode may use private filesystem paths and the optional SQLite adapter.
- Worker mode must store persistent records in operator-selected bindings or a customer-managed database. Worker memory is request/runtime state, not durable storage.
- Container mode must not keep authoritative data only on container disk. Cloudflare Containers use ephemeral disk by default; a restarted/replaced instance can receive a fresh disk.
- Queues and Workflows are delivery/orchestration systems, not substitutes for an application's idempotency and business-state model.
- Every external side effect needs an idempotency key, a recorded operation state, bounded retries and reconciliation for unknown outcomes.

## 5. Headless API and client model

The engine-facing integration surface should be ordinary, versioned protocols:

- HTTPS JSON APIs for request/response operations;
- authenticated webhooks for inbound events;
- explicit queue/event contracts for asynchronous work;
- OAuth or scoped API credentials held by the host/provider runtime, never embedded in browser JavaScript;
- structured health, readiness and version endpoints;
- API versioning and stable error envelopes.

A responsive web console is optional and separate from the core. A user may operate the headless service from a phone browser, but the phone does not need an Otya app, APK or desktop binary.

The host application owns end-user authentication and authorization. The engine must not invent a central Otya user/tenant directory merely because it exposes an HTTPS API.

## 6. Phone-only development and deployment

1. Edit a branch in GitHub's browser editor or a GitHub Codespace.
2. Push the branch from the phone browser.
3. Let GitHub Actions compile and test on hosted runners.
4. Review workflow logs and artifacts from the phone.
5. Start a preview/production deployment through the repository's protected workflow or the operator's chosen build integration.
6. Open the HTTPS endpoint and inspect runtime logs/health from the Cloudflare dashboard or chosen host dashboard.

The repository contains a manual Cloudflare deployment workflow. It requires the operator to configure CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN as GitHub Actions secrets. The workflow has separate preview and production targets; production is restricted to main. It does not contain a token or deploy automatically on every push.

GitHub's workflow_dispatch action is the phone-accessible control. The workflow must first be present on the repository's default branch before it can be manually run from the Actions UI. For this Rust Worker, GitHub Actions is the selected builder because the documented default Cloudflare Workers Builds image lists Node.js, Python, PHP, Ruby and Go but does not list Rust. A future custom build image or updated Workers Builds support can be evaluated separately.

Do not configure two deployment systems to automatically publish the same Worker from the same branch. Choose one deployment owner per Worker.

## 7. Portability is a tested support matrix, not a promise of "everything"

A new target is supported only after all relevant gates pass:

- compile and link for its exact target triple;
- runtime integration tests against the real host API;
- storage consistency and concurrency tests;
- startup, graceful shutdown and restart tests where applicable;
- authentication, permissions and secret-handling tests;
- network failure, timeout, retry and unknown-outcome tests;
- backup/restore and data-preservation tests for persistent storage;
- size, memory, CPU and latency budgets;
- release and rollback procedures.

A Rust core compiling to WebAssembly does not mean the Linux CLI or SQLite crate also runs in a Worker. An Android target compile check does not create an APK or prove a complete Android application. A container image building successfully does not prove its storage survives container replacement.

## 8. Current status

Implemented in this branch:
- portable Rust core with Android and WebAssembly compile checks;
- Linux CLI and optional local SQLite adapter;
- an initial application SDK with manifests, exact routes, public/authenticated policy and capability checks;
- a stateless example catalog application and route tests;
- a Cloudflare Worker serving a responsive example website and routing API calls to the example app;
- a manual, phone-triggerable GitHub Actions deployment workflow requiring operator-owned Cloudflare secrets.

Still incomplete:
- Worker browser/runtime end-to-end tests and a successful first deployed preview;
- real authentication and host authorization adapter; the current sample routes are public/read-only;
- async application handlers, request-body forwarding and general HTTPS host adapter;
- D1/Queues/Workflows/Durable Objects storage and operation adapters;
- generic asynchronous storage/operation contracts;
- Cloudflare Container runtime profile and generic OCI packaging;
- generic HTTP server/service adapter for non-Cloudflare hosts;
- authenticated management API, responsive operator console and first-run setup UI;
- production deployment approval, resource isolation, backup/restore and recovery acceptance.

No standalone Otya Android/iOS application or APK is part of the engine product.
