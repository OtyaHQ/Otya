# Building Independent Applications on Otya Engine

**Status:** operator model and adoption policy; customer-ready distribution is not complete.  
**Reviewed:** 2026-11-28.

## The correct mental model

Otya Engine is software that an application developer or operator installs and uses as a reusable foundation. It is not a hosted Otya account, shared customer platform, business application or website builder.

A person or organization runs its own installation in its own environment. It controls the host, app source, configuration, storage, credentials, network, provider accounts, backups and release process. Another organization runs a separate installation. The installations do not share an Otya customer database and do not register themselves with Otya.

An operator can run many applications inside one Engine installation. The installation provides a local/operator-controlled management plane with workspaces, project and component inventory, environment configuration, shared-service grants, deployment plans, health, backups and audit. Each project remains separately versioned, secured, deployed and recoverable. Projects may consume shared identity, provider adapters, jobs, notifications or observability only through explicit scoped grants. Use separate named installations when a stronger operator or trust boundary is required. This is not a global Otya-hosted control plane or a multi-tenant pool for unrelated customers. See [Multi-project operator architecture](MULTI_PROJECT_OPERATOR_ARCHITECTURE.md).

## How an independent developer builds on it

\`\`\`text
Developer's own Git repository
  ├── web / mobile / desktop frontend
  ├── application domain logic and business rules
  ├── application authentication and authorization policy
  ├── application manifest and route definitions
  └── tests, migrations and release pipeline
              |
              v
       Otya SDK / module contracts
              |
              v
       Otya Engine installation
  ├── portable runtime and capability checks
  ├── configuration and operator paths
  ├── storage adapter selected by operator
  ├── security and operation primitives
  ├── optional modules and provider adapters
  └── diagnostics and lifecycle tooling
              |
              v
       Operator-owned environment
  Linux / container / Cloudflare / other supported host
  Operator's database, object store, mail and provider accounts
\`\`\`

The developer keeps a normal separate application repository. Otya is a dependency/runtime underneath it, not a requirement to move the application source into the Otya repository. A frontend remains a frontend; it calls its application's backend/API. It does not receive database credentials, provider secrets or privileged Engine host bindings.

### The three integration forms

**1. Embedded Rust library — the first supported developer path.**

A Rust app links the Engine SDK in its own process. The app controls startup and shutdown and supplies trusted caller context. No network service is required just to call in-process APIs.

The current SDK crate is otya-engine-sdk. It is still an unpublished workspace crate (publish = false). Today, a developer must work against an Otya source checkout or a local path dependency. A versioned, signed, published SDK package is a release prerequisite before an unrelated external developer can install it cleanly by package version.

**2. Operator-managed local/server service — planned.**

A Python, TypeScript, Flutter, Java, Go or other non-Rust application should eventually connect through a documented local IPC or authenticated HTTPS contract. The Engine service will own the reusable runtime lifecycle; the application will still own its routes, user permissions and business rules.

The current CLI does not provide otya serve, a generic HTTP server, a Unix socket API, a stable C ABI or a published Python/Dart client. Do not invent these commands or assume that a non-Rust app can connect to a production Engine today.

**3. Host-specific deployment adapter — preview/demo stage.**

The current Cloudflare Rust/WebAssembly adapter proves that the portable core can be packaged as a Worker and serve a small read-only catalogue demonstration. It exposes health, version and capability metadata. It does not yet implement authenticated production application traffic, asynchronous storage, request-body forwarding, durable jobs or product authorization.

A Cloudflare customer must deploy into their own account and bind their own resources. A Cloudflare Worker is not a general Linux server: it does not provide arbitrary local disk, process spawning or the full Linux service environment. A Linux host adapter is a separate runtime target.

The .ot language is not yet an executable product language: the repository has a language specification and examples, but no production parser, compiler, package manager or runtime. Developers use the experimental Rust SDK today. The language can later become an additional application authoring frontend that compiles to the same stable Engine contracts; it must not be confused with the Engine runtime itself.

## What is actually usable today

| Capability | Current evidence | Use boundary |
|---|---|---|
| Configuration parsing and validation | Implemented in the Rust core and CLI | Local operator configuration |
| Private config initialization | otya config init; exclusive create, mode 0600 on Linux | Local setup |
| Operator path discovery and private directory preparation | otya paths, explicit --create | Local Linux filesystem |
| Local diagnostics | otya doctor | Reports what the current build can detect; not a production certification |
| Storage contract | Core traits and optional Linux SQLite adapter | Local storage primitives, not a full business database |
| SQLite initialization/check | otya storage init/check | Explicit local database operations |
| Application manifest and exact route registry | Experimental Rust SDK | In-process application prototype |
| Host-supplied caller/capability context | SDK contract | Trust boundary only; not a token verifier |
| Cloudflare Worker package | Wasm build and read-only demo | Preview/package proof, not production app host |

The Engine is therefore an architecture and local-runtime foundation, not yet a finished replacement for production application backends. Its current SDK cannot safely take over OEnd, OPoint, Omy or O because the async host, auth adapter, durable operation and storage interfaces are not complete.

### Current local setup

After installing a compatible Linux build:

\`\`\`sh
otya version
otya doctor
otya paths
otya config init
otya config validate --file <path-shown-by-otya-paths>/otya-engine.toml
\`\`\`

The starter config deliberately selects memory storage and warns that it is volatile. It is not a production profile.

To try the current SQLite adapter, edit the created config to select a file-backed SQLite database with an absolute path inside the operator's data directory:

\`\`\`toml
schema_version = 1

[engine]
mode = "embedded"

[storage]
backend = "sqlite"
path = "/absolute/operator-owned/data/otya-engine/engine.db"
busy_timeout_ms = 5000
\`\`\`

Then run:

\`\`\`sh
otya config validate --file <config-file>
otya storage init --config <config-file>
otya storage check --config <config-file>
\`\`\`

These commands validate and initialize the current local adapter. They do not install a background service, publish a web API, create application tables, provide automatic backups or start business workloads.

For the existing Rust application contract example, see examples/catalog-app and run:

\`\`\`sh
cargo test -p otya-engine-sdk -p otya-example-catalog-app
\`\`\`

That example is intentionally static and read-only. It proves an application can declare a manifest and register routes against the SDK; it is not a deployable customer commerce backend.

## What an installed Engine should eventually provide

These are the planned reusable capabilities that would make an independent installation valuable. Each must be a separately versioned module or adapter, installed only when needed.

| Engine capability | Reusable work it can remove from an app | What the app still owns |
|---|---|---|
| App/runtime lifecycle | Process supervision, configuration loading, readiness, shutdown, module registration | Application entry points and domain behavior |
| Storage adapters | Backend connection, transactions, namespaced storage, migration hooks and health | Business schema, data ownership and domain invariants |
| Durable operations | Idempotency, operation state, outbox/inbox, retry scheduling, leases and reconciliation primitives | Meaning of each operation and which actions are safe to retry |
| Security primitives | Secret resolution interfaces, capability checks, audit hooks, request limits and policy hooks | User identity, roles, business authorization and approval decisions |
| Events and jobs | Queue contracts, scheduler, delivery attempts, dead-letter handling and replay controls | Event meaning and final business-state transitions |
| Integration adapters | HTTP client, signed webhook verification primitives, email/SMS/push/object-store connectors | Provider contract, commercial account, legal consent, credentials and provider-specific decisions |
| Observability | Structured logs, correlation IDs, health, metrics and diagnostics | Product dashboards, support policy and incident ownership |
| Packaging and updates | Signed module/package verification, compatibility checks, upgrade plans and rollback hooks | App releases, customer notices and acceptance |
| Files and media infrastructure | Local/object storage adapter, checksums and streaming utilities | Media rights, file visibility, content moderation and product UX |

The Engine must not claim a provider integration is active or successful merely because an adapter is configured. The adapter must return verifiable provider evidence and report unavailable or unknown states honestly.

## What Otya will not absorb

The Engine must not become the source of truth for every application's domain. A general engine can supply reliable primitives; it cannot infer the correct rules for every business.

- It does not decide whether a payment, tax treatment, API use or provider workflow is legally permitted.
- It does not create or own an application's end-user accounts, companies, branches, employees, roles or customer relationships.
- It does not replace the application's authorization policy or approval chain.
- It does not invent the product UI, mobile experience, media catalogue or business workflows.
- It does not own a product's financial ledger, inventory, customer data, content rights or provider credentials.
- It does not make external providers, networks, servers or Cloudflare available.
- It does not require central Otya telemetry, account registration, a vendor-controlled database or a phone-home service.
- It does not guarantee that every host can implement every capability. Each host adapter declares its actual capabilities and limitations.

## Otya's own products must be treated like independent Engine customers

The first-party products are validation consumers, not a reason to add Otya-specific exceptions to the engine. They should integrate through the same documented package, SDK and host contracts that an unrelated developer receives.

- **OEnd** consumes generic runtime, operation and event capabilities while remaining the API gateway and product integration router.
- **OPoint** consumes approved generic storage/operation/audit capabilities but retains company/branch authorization, inventory, cash, provider float, reconciliation and double-entry financial truth.
- **Omy Play** may consume connected-service, release, update and asset metadata capabilities, but local playback, local media, private vault and device-owned library remain independent of cloud availability.
- **O** may consume runtime, task and observability capabilities, but its owner identity, memory, approval policy and privileged actions remain in O's application boundary.
- **OID** remains the identity authority. Engine capability context is not a replacement for OID token/session verification.
- **OBackup** remains the separately controlled recovery path until an Engine backup module has been proven against all databases, objects, encryption keys, retention and restore requirements.
- **OWeb** remains the public frontend. It uses OEnd as its server-side gateway; it does not expose Engine internals to browsers.
- **OHub** documents the supported Engine SDK and modules; it does not need to run an Engine instance.

No first-party application should receive privileged behavior merely because it belongs to Otya. It must pass the same conformance, permission, failure and recovery tests as an unrelated application.

## Repository and Worker retirement decision

### GitHub repositories

The connected organization currently has these repositories:

| Repository | Decision | Reason |
|---|---|---|
| OtyaHQ/Otya | Keep | Engine source, SDK, CLI, runtime adapters and release artifacts |
| OtyaHQ/OEnd | Keep | Public API gateway, BFF, integration router and release coordination |
| OtyaHQ/OPoint | Keep | Independent business product and financial authority |
| OtyaHQ/Omy | Keep | Independent consumer media product and connected-service backend |
| OtyaHQ/O | Keep | Private owner application and AI workflows |
| OtyaHQ/OWeb | Keep; build it | Public website/frontend source; currently an empty repository |
| OtyaHQ/OHub | Keep | Public developer/partner documentation |
| OtyaHQ/.github | Keep | Organization profile and shared GitHub defaults |

The connected organization has no separate OBackup or OID repository in this inventory. Their current Worker implementations are maintained in the existing OEnd repository. Do not create duplicate repositories simply to mirror each Worker.

The old Omy_Play spelling is a historical repository reference. The canonical source repository is OtyaHQ/Omy; update stale source links and workflow references after checking branch/tag history. This is not a reason to delete the canonical Omy repository.

### Deployed Cloudflare Workers

Read-only inventory of the connected account found six deployed Workers:

| Worker | Decision | Current unique responsibility |
|---|---|---|
| oend | Keep | Public API gateway, BFF, OID/Omy/OPoint/O service bindings, queues and release workflow |
| oid | Keep | Identity, sessions, recovery, authentication security and its own auth database |
| opoint | Keep | OPoint business API, OID integration and OPoint-owned data/secrets |
| omy | Keep | Omy connected-service API, Omy D1/R2, release/device/diagnostics and Firebase integration |
| o | Keep | Owner application/agent, owner Durable Object, AI bindings and O-owned state |
| obackup | Keep | Separate backup workflow, access to product databases and backup object stores |

The service bindings and data resources are evidence that these Workers are not duplicate copies of one generic Engine Worker. Their distinct failure, security, data and release boundaries are intentional.

The new otya-engine-edge and otya-engine-edge-preview names are Engine host-adapter targets, not replacements for those six product Workers. The production Engine profile is private by default; the preview profile is isolated and demonstration-only.

### What may be retired later

- OEnd's public website/static-page code may be extracted to OWeb after route, session, cache, SEO, accessibility, analytics and rollback parity are proven. OEnd itself remains the API gateway.
- Repeated generic utilities may move into shared Engine modules after the module passes conformance tests and product owners accept the dependency.
- A product-specific Worker may be reduced only if its business rules and integration behavior have genuinely moved to an independently deployable product service with the same data and security guarantees. Engine reuse alone is not sufficient.
- An old compatibility route or historical source link may be removed after client usage and deployment references are checked.

### Deletion gate

Do not delete a Worker, repository, database, queue, bucket, DNS route or secret as a cleanup shortcut. Before a resource is retired:

1. Identify its exact source repository, deployment workflow, callers, service bindings, hostnames, queues, secrets and data resources.
2. Prove the replacement implementation and contract tests cover all supported clients and failure cases.
3. Export and verify any data that must be retained; record retention and legal requirements.
4. Run the replacement in isolated staging and compare behavior with the old implementation.
5. Move traffic through a reviewed, reversible change and observe logs/telemetry for old-client usage.
6. Preserve a rollback window and restore evidence.
7. Remove only the specifically approved resource after owner review.

**Current deletion recommendation: none.** Keep the six deployed Workers and all eight repositories. The immediate work is to make Otya genuinely useful as an installable SDK/runtime and then migrate shared plumbing selectively—not to delete functioning product boundaries.

## Release gates before external customer adoption

1. Publish a semver-versioned SDK and host contract with a compatibility policy.
2. Implement a safe app package manifest, signed module verification and dependency resolution.
3. Implement asynchronous host/storage contracts and at least one durable local backend.
4. Add a production Linux service/HTTP host with authentication delegated to a trusted application host.
5. Implement durable jobs, event/outbox/inbox, retries, leases, scheduling and reconciliation.
6. Add database migrations, backup/restore, crash recovery and fault-injection evidence.
7. Add a real app scaffold command and a separate example repository that builds using only released artifacts.
8. Add Python and TypeScript clients only after the wire protocol is stable; add Dart/Flutter embedding only after a reviewed FFI ABI.
9. Test upgrade, rollback, uninstall and data preservation on clean Linux installations.
10. Run Otya's own products as ordinary consumers of those same released contracts.

Until these gates pass, describe the Engine as experimental and do not advertise the planned modules or host modes as shipped.
