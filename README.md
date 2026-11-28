# Otya Engine

**Your platform. Your infrastructure. Your control.**

**Otya is an independently installable, customer-operated application platform and systems engine.**

Otya is installed and operated independently by the person or organization choosing to run it. Each installation owns its identity realm, user directory, signing keys, workspaces, projects, service configuration, data, provider accounts and deployment credentials. It is not a shared multi-tenant service: unrelated installations do not share users, services, data or control planes, and no Otya-hosted account or remote heartbeat is required.

The Rust core remains headless and portable, while the complete installation includes a local operator control plane and browser-based management console. An operator can use the UI without a terminal or use the CLI, SDK and APIs for development and automation. Workloads can run embedded, as local/server services, or through host adapters such as Cloudflare Workers. Otya is not a mandatory hosted SaaS or an Android/iOS app-store client. See [Private platform distribution](docs/PRIVATE_PLATFORM_DISTRIBUTION.md).

## What the engine provides

- A reusable Rust core with stable, versioned interfaces.
- An embeddable library for applications, with native bindings where needed.
- An operator-controlled service supervisor, management API and optional local IPC.
- A browser-based operator console for non-developers, with guided setup, access management, service configuration, monitoring and deployments.
- An installation-owned identity provider and authorization foundation, with local user/group management, sign-in, MFA, sessions, OIDC/OAuth and optional external federation.
- A reusable UI kit, design system, templates and schema-driven application forms.
- A CLI and installation kit for setup, configuration, health checks, upgrades, backup and recovery.
- Pluggable storage, identity, authorization, messaging, filesystem, network and provider adapters.
- Reusable business and operational modules with explicit permissions.
- Durable operations: idempotency, queues/outbox, retries, reconciliation and audit history.
- Local diagnostics and observability with privacy-preserving defaults.

The engine does not silently create a central Otya service or send customer data to Otya. Remote integrations are opt-in and configured with accounts and credentials controlled by the operator. The built-in identity service, management UI and shared modules are local to that operator's installation; they are not shared with other Otya customers.

**One installation can manage many projects.** An individual developer or company can install Otya once, create workspaces, register multiple applications and deployment components, and explicitly grant selected projects access to shared capabilities. The operator gets one local management plane for inventory, configuration, secrets grants, deployments, health, backups and audit, while each application keeps its own source, data authority, authorization, runtime and release lifecycle. This is local/operator-controlled management, not a global Otya-hosted service or a cross-customer tenant pool. See [Multi-project operator architecture](docs/MULTI_PROJECT_OPERATOR_ARCHITECTURE.md).

## Applications and websites built on Otya

Otya is the foundation developers build on, not one monolithic application. The experimental `otya-engine-sdk` introduces application manifests, API routes, host-supplied authentication context and capability checks. The example catalog application and its responsive website demonstrate how a separate frontend calls an Otya-powered API. The example is stateless demonstration data, not a production commerce system. See [Applications on Otya](docs/APPLICATIONS_ON_OTYA.md), [example catalog app](examples/catalog-app/README.md) and [Cloudflare Worker host adapter](platforms/cloudflare-worker/README.md).

## Build as an independent operator

A developer keeps an application in their own repository and uses Otya through a local installation, released SDK or versioned HTTPS/JSON contract. Rust is the core implementation language, not a restriction on application authors: any language with a supported SDK or HTTP client can build on top. Every operator controls their own installation, data, provider accounts and infrastructure. The current SDK is experimental and unpublished; the external service API and application scaffolder are not shipped yet. See [Building Independent Applications on Otya](docs/INDEPENDENT_OPERATOR_GUIDE.md) for what works today, the planned integration forms, and the audited repository/Worker retirement decisions.

## The Media Kit design lesson

Omy Play uses the third-party media_kit package as an embedded playback engine. Its modular packaging separates the shared player API from optional video rendering and native media libraries. Otya should learn from this packaging pattern:

- one stable public API over a shared core;
- optional components installed only when needed;
- small platform-specific bridges around a shared implementation;
- no requirement for a separate vendor-hosted service to perform local work.

Otya is not a media player and does not copy Media Kit's code. We are adopting the architectural lesson of a reusable, installable kit. See [Media Kit design lessons](docs/MEDIA_KIT_DESIGN_LESSONS.md).

## Operating modes

1. **Embedded mode (primary):** a host application links the Otya Rust library through Rust APIs or supported FFI bindings. No daemon or public network listener is required.
2. **Per-user service mode (optional):** the installing OS user runs one local Otya service for that user's trusted applications. Local IPC is owner-only by default. It is not a shared service for unrelated OS users; another operator installs a separate instance.
3. **Operator-managed server mode (optional):** an operator may run the same engine on a server under a dedicated, unprivileged service account. That account operates one explicitly configured installation for that operator's deployment. It does not create a shared tenant pool or serve unrelated operators through engine-managed customer spaces.

All modes share the same portable core and versioned contracts. The Cloudflare Worker adapter is a separate Wasm host wrapper; the native Linux CLI/SQLite adapter is not assumed to run inside Workers. See [Headless and Portable Runtime Model](docs/HEADLESS_AND_PORTABLE_RUNTIME.md).

## Data and infrastructure ownership

Otya does not provide mandatory hosted compute, cloud storage or a central database.

- The host application may provide storage through a versioned storage interface.
- The portable core defines a backend-neutral storage contract; a separate optional Linux-only SQLite adapter implements it. The Linux CLI includes optional SQLite storage commands; customer-managed PostgreSQL remains planned.
- Files can remain on the local filesystem or use an operator-selected object-storage adapter.
- Credentials are resolved through the operator's operating-system secret store, environment/credential injection or an explicitly configured vault.

The SQLite adapter implements the core storage contract: namespaced byte-key/value operations, explicit commit with rollback on drop, bounded prefix scans, WAL, private database files, a v1-to-v2 schema upgrade and non-creating current-schema checks. The full integration passed Otya Engine CI [run 265](https://github.com/OtyaHQ/Otya/actions/runs/46143575958), including stable formatting, Clippy, tests, CLI storage smoke tests, release build and Rust 1.85 checks. Business-module schemas, a general migration runner, online backup/restore, crash recovery and stress/fault-injection testing remain incomplete. The starter config still uses memory storage and warns that it is volatile. This branch is experimental; do not put production data into it. See [SQLite adapter status and requirements](docs/STORAGE_SQLITE.md).

## Bring your own infrastructure

Otya does not host user applications, databases, object storage, email, payment services or cloud resources. Operators connect accounts and resources they already own. Otya Connect is the planned provider-connection layer for Cloudflare, Firebase, Supabase, SMTP/Resend, payment, AI, storage and custom REST/webhook services. It stores credentials in the operator's own installation vault, checks scopes, maps provider capabilities and prepares deployments with explicit approval. Operators pay providers directly. See [Bring-your-own-infrastructure provider architecture](docs/BYI_PROVIDER_CONNECTIONS.md).

A provider connection does not mean Otya is hosting that provider or that all providers are interchangeable. Use a local Linux adapter when the operator wants a local service. Where an app runs inside a provider runtime, use that provider's native bindings and secret facilities where supported. Already deployed apps must not depend on the management console being online.

## Provider integrations

Cloudflare, GitHub, Resend and Firebase are optional adapters, not engine dependencies:

| Provider | Optional responsibility |
|---|---|
| Cloudflare | Operator-authorized edge deployment and services such as Workers, R2 or D1 |
| GitHub | Source control, release coordination and approved Actions workflows |
| Resend | Transactional email delivery and verified delivery events |
| Firebase | Mobile push and selected client-integrity or identity integrations |

Generic HTTP, signed webhooks, SMTP and S3-compatible storage are additional adapter categories. Each adapter must declare its permissions, required configuration, data sent, health checks, retry rules, provider limitations and failure behavior. No adapter may claim a provider operation succeeded without provider evidence.

## What Otya is not

- Not a hosted SaaS backend.
- Not a cloud provider, hosted database or storage vendor. Otya can run local services and manage resources in accounts the operator owns.
- Not a replacement for Linux or the operator's operating system.
- Not a shared, Otya-hosted identity provider; each installation may run its own private identity service.
- Not a programming language, compiler or language runtime.
- Not a guarantee that third-party APIs, providers or operator infrastructure are available or compliant.

## Development status

This repository is the source of truth for Otya Engine. The current repository is an experimental architecture foundation; architecture documents are not evidence that an installer, daemon, adapter or production service already exists.

Every feature must be labeled accurately as proposed, implemented, CI-tested, manually verified or released. See [Engine roadmap](docs/ROADMAP.md), [requirements register](docs/ENGINE_REQUIREMENTS_REGISTER.md), [comprehensive gap audit](docs/ENGINE_GAP_AUDIT.md), and [implementation gates](docs/ENGINE_RELIABILITY_MODEL.md).

## Phone-first development and headless operation

A contributor or operator can use a phone browser with GitHub, Codespaces and GitHub Actions doing the heavy editing, compilation, testing and deployment work remotely. Phone users access Otya-powered products through the host application's responsive website/mobile client or a headless HTTPS API. The engine itself is not distributed as an APK or mobile app. Android compile checks only preserve future host-application embedding options. See [Phone-First Development](docs/PHONE_FIRST_DEVELOPMENT.md), [Mobile Runtime Model](docs/MOBILE_RUNTIME_MODEL.md) and [Headless and Portable Runtime Model](docs/HEADLESS_AND_PORTABLE_RUNTIME.md).

## Engineering influences

Otya studies mature systems and adopts proven engineering practices without copying their identities or rebuilding their entire products. See [Engineering influences and design rules](docs/ENGINEERING_INFLUENCES.md).

## Security, licensing and contributions

The engine must use deny-by-default capabilities, least privilege, signed release artifacts, dependency review, bounded resource use and privacy-conscious diagnostics. See [Security policy](SECURITY.md).

Repository visibility does not by itself grant permission to reuse, redistribute or modify the software. The license, trademark rules and contributor terms must be reviewed and approved before external code contributions are accepted. See [Contributing](CONTRIBUTING.md).

## Start here

- [Engine architecture](docs/ARCHITECTURE.md)
- [Otya brand and product identity](docs/BRAND_AND_PRODUCT_IDENTITY.md)
- [Bring-your-own-infrastructure provider connections](docs/BYI_PROVIDER_CONNECTIONS.md)
- [Otya brand and product identity](docs/BRAND_AND_PRODUCT_IDENTITY.md)
- [Bring-your-own-infrastructure provider connections](docs/BYI_PROVIDER_CONNECTIONS.md)
- [Private platform distribution: identity, UI and integrated services](docs/PRIVATE_PLATFORM_DISTRIBUTION.md)
- [Multi-project operator architecture](docs/MULTI_PROJECT_OPERATOR_ARCHITECTURE.md)
- [Installable kit and packaging model](docs/INSTALLATION_KIT.md)
- [Configuration lifecycle and safe initialization](docs/CONFIGURATION_LIFECYCLE.md)
- [Optional SQLite storage adapter](docs/STORAGE_SQLITE.md)
- [Phone-first development](docs/PHONE_FIRST_DEVELOPMENT.md)
- [Mobile runtime model](docs/MOBILE_RUNTIME_MODEL.md)
- [Headless and portable runtime](docs/HEADLESS_AND_PORTABLE_RUNTIME.md)
- [Applications and websites on Otya](docs/APPLICATIONS_ON_OTYA.md)
- [Example catalog application](examples/catalog-app/README.md)
- [Operator and ownership model](docs/OPERATOR_MODEL.md)
- [One installation, one operator](docs/ONE_INSTALL_ONE_OPERATOR.md)
- [Linux platform model](docs/LINUX_PLATFORM_MODEL.md)
- [Engineering influences](docs/ENGINEERING_INFLUENCES.md)
- [Comprehensive gap audit](docs/ENGINE_GAP_AUDIT.md)
- [Reliability, backup and recovery](docs/ENGINE_RELIABILITY_MODEL.md)
- [Modules and provider adapters](docs/MODULES_AND_ADAPTERS.md)
- [Media Kit design lessons](docs/MEDIA_KIT_DESIGN_LESSONS.md)
- [Requirements register](docs/ENGINE_REQUIREMENTS_REGISTER.md)
- [Roadmap and acceptance gates](docs/ROADMAP.md)
