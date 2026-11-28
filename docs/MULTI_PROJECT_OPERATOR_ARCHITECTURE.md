# Multi-project operator architecture

**Status:** target architecture and implementation specification; the control plane and workspace orchestration described here are not implemented yet.  
**Reviewed:** 2026-11-28.

## Decision

Otya Engine must support **one independently installed Engine installation managing many projects for one operator**, with explicit, permissioned sharing of reusable services.

The product invariant is:

> One installation belongs to one operator boundary. That operator may manage many applications, services, repositories, environments and teams inside it. Applications remain separately owned and deployable. Sharing is explicit and policy-controlled.

This is not a global Otya-hosted control plane and not a public multi-tenant SaaS pool. An unrelated developer or company installs and controls a different Engine installation. They do not register with, depend on, or share data with another operator's installation.

The previous "one installation, one operator" rule remains correct; it must not be misread as "one installation, one application" or "no local control plane". The Engine should have a local/operator-controlled control plane for that operator's own estate.

The complete installable distribution should also include an operator console, installation-owned identity provider/login portal, authorization service, reusable UI kit and first-party shared service catalogue. These are local to the installation. See [Private platform distribution](PRIVATE_PLATFORM_DISTRIBUTION.md) for the identity, UI, non-developer and installation-isolation design.

## The four levels

1. **Installation / operator boundary** — one Engine installation controlled by an individual developer or organization. It owns the workspace registry, operator policies, provider connections, deployment credentials and management state.
2. **Workspace** — a group of related projects managed together, such as a company's product ecosystem. A workspace can define reusable services, access grants and environment policies.
3. **Project** — an independently versioned application or service with its own repository or source package, ownership, configuration, data authority, secrets namespace, release history and lifecycle.
4. **Component / deployment unit** — a deployable part of a project: a Worker, Linux service, frontend, scheduled task, queue consumer, mobile API or other workload. One project can contain several components and one repository can build more than one component.

Environments (development, staging and production) are separate deployment scopes under a project. They must not silently share production credentials or mutable data.

## Logical architecture

~~~text
Operator (developer / company)
        |
        v
One locally installed Otya Engine
        |
        +-- Operator control plane
        |     +-- workspaces and project registry
        |     +-- repository / artifact references
        |     +-- environments and release history
        |     +-- configuration and policy versions
        |     +-- secret references and grants
        |     +-- deployment and rollback plans
        |     +-- central health, logs, metrics and audit views
        |
        +-- Shared capability services (opt-in, scoped)
        |     +-- identity-provider / SSO integration
        |     +-- email, push and notification adapters
        |     +-- integration and webhook adapters
        |     +-- job scheduling and event delivery
        |     +-- secrets broker / credential references
        |     +-- backup coordination and restore verification
        |     +-- observability and audit collection
        |
        +-- Project registry
              +-- OEnd project
              |     +-- OEnd API gateway component
              |     +-- OID identity component
              |     +-- OBackup recovery component
              +-- OPoint project
              |     +-- OPoint application/API components
              +-- Omy project
              |     +-- Omy media API component
              +-- OWeb project
              |     +-- public/private website components
              +-- O project
                    +-- owner assistant components

Workloads may execute on Linux, Cloudflare Workers or other supported hosts.
The controller manages them; it does not require all workloads to run in one process.
~~~

The project/component mapping above is a proposed management view of the current estate, not a change to repository ownership or deployed resources. OID and OBackup are currently deployment units whose source is maintained with OEnd; they are not assumed to be new repositories.

## What should be shared

Sharing is a first-class Engine feature. It should be declared, permissioned, observable and revocable.

| Capability | Recommended sharing model |
|---|---|
| Staff identity / SSO | Prefer the installation-owned Otya identity issuer and login portal for company staff; optionally federate to an external OIDC/SAML provider. Each application retains its own roles, permissions and authorization rules. |
| Public/customer identities | Separate by product by default. Omy listeners, customers or other public users must not automatically become OPoint or owner-app users. |
| Email and push delivery | Shared operator-managed provider integration, with per-project sender identities, templates, scopes, quotas and credentials where supported. |
| Provider connections | A centrally registered connection may be granted to selected projects. Prefer distinct least-privilege keys per project; never expose the raw provider secret to frontends. |
| Secrets | One operator-controlled secret store/broker may manage secrets, but project namespaces and explicit read/use grants are mandatory. A project sees references and receives only the value it is authorized to use. |
| Jobs and events | Shared scheduler or broker infrastructure may serve many projects, but each project gets isolated queues/topics, access policies, retry limits and dead-letter handling. |
| Files and object storage | A shared storage service/cluster may be used, but buckets, prefixes, encryption keys and access policies are isolated per project unless a cross-project share is explicitly approved. |
| Observability | Central dashboards and correlated operational views are desirable. Application data and secrets must be redacted; access to logs remains project-scoped. |
| Backups | One backup coordinator can manage schedules and verification across projects. Backup destinations, encryption keys, retention and restore permissions remain separately controlled. |
| Deployment operations | One control plane can plan and track releases for many projects. Each project has its own artifact, environment, credentials, rollout and rollback boundary. |
| Reusable code | Versioned Engine modules and SDK packages are shared as dependencies, not by merging product repositories or copying implementation between apps. |

A shared provider connection or service does not imply shared application data. A shared storage cluster does not imply a shared database schema or unrestricted database access.

## What must remain independent

Each project must retain:
- its own source repository or clearly versioned source package;
- its own product/domain logic and data authority;
- its own runtime component and release lifecycle;
- its own environment configuration and project-scoped secret grants;
- its own authorization rules and API contract;
- its own database, schema or isolated storage namespace by default;
- its own deployment, rollback and recovery decisions.

Cross-project data access should happen through an authenticated API or an explicitly governed data contract. Directly sharing database credentials or allowing one app to read another app's tables is prohibited by default.

A shared company staff identity can provide SSO, but it must not collapse application authorization. For example, being authenticated as a company employee does not automatically grant the person access to OPoint finance, O's owner functions or Omy moderation.

## Control plane versus workload plane

The Engine control plane is the operator's management layer and visual console for developers and non-developers. It stores desired state, inventories projects and components, manages the installation-owned identity portal, checks policies, prepares deployment plans, records outcomes and coordinates providers through adapters.

The workload plane is where application code actually runs. It may be:
- embedded in a Rust application;
- hosted as an isolated Linux service or process;
- deployed as one or more Cloudflare Workers;
- hosted on another supported provider.

The control plane must not be a single point of runtime failure. If the local management host is offline, already deployed applications should continue operating. Workloads should only need the control plane for explicitly designed management operations, not for every request.

For Cloudflare, the Engine should be able to manage a fleet of separate Workers through an adapter and the operator's Cloudflare account. It must not collapse all Workers into one merely to make the dashboard look simpler. Service Bindings and other private provider mechanisms can connect selected components where justified.

## Otya company workspace: recommended first mapping

Use one operator installation and one company workspace, with separate projects and environment boundaries:

| Project | Current responsibility | Keep independent | Candidate shared capabilities |
|---|---|---|---|
| OEnd | Central ecosystem backend, API gateway and cross-product orchestration | API routes, product integration logic, gateway release; OID and OBackup remain separately deployable components | Engine runtime contracts, observability, secrets references, deployment control, approved shared adapters |
| OPoint | Private business application | Business data, finance, branch permissions, business rules and release | Selected staff SSO, notifications, audit primitives, backup coordination |
| Omy | Public media application | Public consumer experience, media catalog, local playback/offline behavior, product data and release | Notifications, media/file infrastructure where appropriate, observability, backup coordination |
| OWeb | Public and private website system | Website frontend, content and web release | Shared identity for private sections, approved APIs, email and observability |
| O | Personal assistant | Owner data, privileged tools, assistant policies and release | Approved identity integration, secrets broker, job runtime and observability |

OEnd remains the Otya ecosystem's central backend. Otya Engine does not replace OEnd's product-specific orchestration or business contracts. Engine provides reusable primitives and the operator control plane. A capability should move out of OEnd only when it is genuinely generic, has a stable Engine contract, passes independent conformance tests and can be consumed by OEnd and an unrelated sample application without Otya-specific assumptions.

## How this solves repositories and Workers

Do not merge repositories as a substitute for central management.

The Engine workspace should connect to existing repositories and record which components each repository builds. One repository may produce several deployable components; multiple repositories may form one product. The Engine should provide a single project inventory, release view and deployment workflow across them.

Likewise, a Worker is a deployment unit, not automatically a project. The Engine can manage many Workers under their owning project, with separate bindings, secrets, routes, database bindings and rollback histories. Retire a Worker only after replacement parity, data migration, security review, backup/restore verification, traffic cutover and rollback validation.

Documentation, shared GitHub organization defaults and source repositories are also manageable assets, but they are not necessarily running application projects. OHub and the organization .github repository remain support assets rather than runtime services.

## Desired declarative workspace model

The long-term management contract should be declarative and versioned. The .ot format may become the human-authored workspace and project manifest, but this is a design target only: the current repository does not yet contain an executable .ot parser or compiler.

Illustrative shape (not currently executable):

~~~text
workspace "otya-company" {
  projects = ["oend", "opoint", "omy", "oweb", "o"]

  shared_service "company-identity" {
    provider = "oid"
    grant = ["oend", "opoint", "oweb", "o"]
  }

  shared_service "mail" {
    provider = "resend"
    grant = ["oend", "opoint", "oweb"]
    credential_scope = "per-project"
  }
}

project "opoint" {
  repository = "OtyaHQ/OPoint"
  isolation = "separate-runtime-and-data"
  environments = ["development", "staging", "production"]
  uses = ["company-identity", "mail", "audit", "backup"]
}
~~~

The real manifest must support schema versioning, validation, dependency references, project-scoped grants, policy checks, environment overlays, secrets references (never plaintext secrets), and a safe plan/apply workflow. Configuration should be reviewable in Git and changes should be diffable before application.

## Implementation sequence

1. **Correct the product model and docs.** Make "one installation, one operator, many projects" explicit. Preserve the no-global-SaaS/no-cross-operator-sharing rule.
2. **Build the local workspace registry.** Add operator/workspace/project/component/environment identities and a read-only inventory before any deployment mutation.
3. **Add declared shared-service contracts.** Start with provider references, identity integration boundaries, secrets grants, observability and backup policies. Add policy evaluation and audit before enabling sharing.
4. **Add host/deployment adapters.** Manage local Linux services first, then Cloudflare Workers and other providers using explicit operator credentials and dry-run plans.
5. **Pilot with non-production Otya components.** Import the existing repositories and Workers as inventory records. Do not take over production secrets, traffic or data during the inventory phase.
6. **Prove isolation and sharing.** Test that an authorized project can use a shared capability, an ungranted project cannot, public users remain product-scoped, and one project's failure or rollback does not stop another.
7. **Migrate incrementally.** Move generic capabilities behind Engine contracts only after parity and rollback evidence. Keep OEnd as the product backend and keep project-specific logic with its product.

## Required acceptance tests

- One Engine installation can inventory and manage at least five projects and multiple components per project.
- A project can consume a shared service only when a valid grant exists.
- Revoking a grant prevents new use without exposing another project's credentials.
- A project's secrets, databases, queues, files and logs are isolated from other projects by default.
- One project's configuration change, deployment failure or rollback does not mutate another project.
- Central staff authentication can be shared while product-specific roles remain separate.
- Public Omy users do not acquire identities or permissions in private OPoint or O.
- Deployed workloads continue operating when the local control plane is unavailable.
- A different developer's separate Engine installation has no connection to this workspace unless they explicitly configure one.
- Workspace inventory and planning work without any live production changes.

## Current implementation status

The current Rust workspace has an early local CLI, configuration and SQLite storage foundations, plus an experimental SDK and preview Cloudflare adapter. It does **not** yet provide the workspace registry, shared-service control plane, multi-project manifest loader, secret broker, deployment orchestrator, generic service daemon or production-grade remote host adapters described above. It also does **not** yet provide the full browser operator console, built-in identity provider/login portal, production authorization service, UI kit or non-developer visual workflows. These are target capabilities specified in [Private platform distribution](PRIVATE_PLATFORM_DISTRIBUTION.md).

This document defines the target; it does not claim those components already exist. No production Workers, repositories, databases, credentials, routes or deployments are changed by documenting this architecture.
