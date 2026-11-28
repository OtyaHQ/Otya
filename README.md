# Otya Engine

**Otya is an independently installable, customer-operated systems engine and reusable integration kit.**

Otya is installed and operated independently by the person or organization choosing to run it. Each installation belongs to its operator and its own local security boundary. It is not a shared multi-tenant service: the engine has no company registration, tenant IDs, end-user directory, central customer registry or remote installation heartbeat. The host application owns its own users and business identities. The operator owns the machine, operating system, data, provider accounts, credentials, backups, network and configuration.

The engine is built in Rust. Applications can embed its libraries or use its optional local service and operator CLI. It must remain useful without an Otya cloud account or a connection to Otya infrastructure.

## What the engine provides

- A reusable Rust core with stable, versioned interfaces.
- An embeddable library for applications, with native bindings where needed.
- An optional local service for background work and local IPC.
- A CLI and installation kit for setup, configuration, health checks, upgrades, backup and recovery.
- Pluggable storage, identity integration, messaging, filesystem, network and provider adapters.
- Reusable business and operational modules with explicit permissions.
- Durable operations: idempotency, queues/outbox, retries, reconciliation and audit history.
- Local diagnostics and observability with privacy-preserving defaults.

The engine does not silently create a central Otya service or send customer data to Otya. Remote integrations are opt-in and configured with accounts and credentials controlled by the operator.

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

All modes share the same engine core and versioned contracts.

## Data and infrastructure ownership

Otya does not provide mandatory hosted compute, cloud storage or a central database.

- The host application may provide storage through a versioned storage interface.
- The portable core defines a backend-neutral storage contract; a separate optional Linux-only SQLite adapter implements it. The Linux CLI includes optional SQLite storage commands; customer-managed PostgreSQL remains planned.
- Files can remain on the local filesystem or use an operator-selected object-storage adapter.
- Credentials are resolved through the operator's operating-system secret store, environment/credential injection or an explicitly configured vault.

The SQLite adapter implements namespaced key/value storage, explicit rollback-safe transactions, bounded prefix scans, WAL, private database files and an initial schema v1-to-v2 migration. The newer generic storage integration still needs its own green CI run. Business-module schemas, a general migration runner, online backup/restore, crash recovery and stress/fault-injection testing remain incomplete. The starter config still uses memory storage and warns that it is volatile. This branch is experimental; do not put production data into it. See [SQLite adapter status and requirements](docs/STORAGE_SQLITE.md).

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
- Not a cloud provider or serverless platform.
- Not a database service.
- Not a replacement for Linux or the operator's operating system.
- Not an identity provider by default.
- Not a programming language, compiler or language runtime.
- Not a guarantee that third-party APIs, providers or operator infrastructure are available or compliant.

## Development status

This repository is the source of truth for Otya Engine. The current repository is an experimental architecture foundation; architecture documents are not evidence that an installer, daemon, adapter or production service already exists.

Every feature must be labeled accurately as proposed, implemented, CI-tested, manually verified or released. See [Engine roadmap](docs/ROADMAP.md), [requirements register](docs/ENGINE_REQUIREMENTS_REGISTER.md), [comprehensive gap audit](docs/ENGINE_GAP_AUDIT.md), and [implementation gates](docs/ENGINE_RELIABILITY_MODEL.md).

## Phone-first development and mobile use

Otya development is designed so a contributor can use a phone browser with a remote Codespaces environment and GitHub Actions doing the heavy compilation and testing. A separate Android-first application architecture is planned so end users can use Otya-powered apps on phones without installing a PC toolchain. The current branch has a Codespaces devcontainer and Android/WebAssembly core compile workflow; these do not yet produce a mobile app or APK. See [Phone-First Development](docs/PHONE_FIRST_DEVELOPMENT.md) and [Mobile Runtime Model](docs/MOBILE_RUNTIME_MODEL.md).

## Engineering influences

Otya studies mature systems and adopts proven engineering practices without copying their identities or rebuilding their entire products. See [Engineering influences and design rules](docs/ENGINEERING_INFLUENCES.md).

## Security, licensing and contributions

The engine must use deny-by-default capabilities, least privilege, signed release artifacts, dependency review, bounded resource use and privacy-conscious diagnostics. See [Security policy](SECURITY.md).

Repository visibility does not by itself grant permission to reuse, redistribute or modify the software. The license, trademark rules and contributor terms must be reviewed and approved before external code contributions are accepted. See [Contributing](CONTRIBUTING.md).

## Start here

- [Engine architecture](docs/ARCHITECTURE.md)
- [Installable kit and packaging model](docs/INSTALLATION_KIT.md)
- [Configuration lifecycle and safe initialization](docs/CONFIGURATION_LIFECYCLE.md)
- [Optional SQLite storage adapter](docs/STORAGE_SQLITE.md)
- [Phone-first development](docs/PHONE_FIRST_DEVELOPMENT.md)
- [Mobile runtime model](docs/MOBILE_RUNTIME_MODEL.md)
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
