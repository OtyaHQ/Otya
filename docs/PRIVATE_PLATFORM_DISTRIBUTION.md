# Private full-platform distribution: identity, UI and integrated services

**Status:** target product architecture and implementation specification; most services described below are not yet implemented in the Engine.  
**Reviewed:** 2026-11-28.

## 1. Product decision

Otya Engine should be distributed as a **complete, self-hosted application platform**, not only as a Rust library or a collection of developer APIs.

One individual or organization installs and operates its own Otya environment. That installation can include:
- a local operator control plane and visual management console;
- an installation-owned identity provider, login portal and access-policy service;
- reusable UI components, design system and application templates;
- reusable application services such as storage, files, events, jobs, notifications, audit, secrets and backups;
- SDKs, CLI, app builder, module manager and deployment tooling;
- host adapters that manage workloads on Linux, Cloudflare and other supported platforms.

The installation is private to its operator. It is not registered into an Otya-operated shared customer platform. Another company or independent developer installs a separate Engine environment with its own identity realm, databases, keys, users, services, domains, provider accounts, configuration and deployments.

The core invariant is:

> One Engine installation = one operator-controlled environment. Inside that environment, one company or individual can run many projects, use shared platform services and administer them through one interface. Separate operators do not share users, services, data or control planes.

This supersedes any interpretation that one installation can manage only one application. It does not create a public multi-tenant service.

The full profile is a **modular distribution**, not one oversized executable. Keep the portable Rust core minimal and measure its binary separately from the supervisor, Console assets, identity module, provider adapters, storage drivers and optional UI builder. A developer embedding only the core should not download the full operator interface. A non-developer installing the full platform gets the complete local Console and selected services; that bundle will have a different size and resource budget.

## 2. The complete installed product

The installable distribution has six cooperating layers.

### A. Engine core and supervisor

The Rust core supplies portable contracts, module lifecycle, configuration, permission checks, resource limits, health interfaces and stable APIs.

A local supervisor starts, stops and monitors Engine services. Modules run with explicit permissions and resource boundaries. Modules must be independently versioned, enabled, disabled, upgraded and rolled back.

### B. Operator control plane and visual console

Every installation receives a browser-based management UI. It is served locally by the installation and can be exposed on a domain controlled by the operator.

It should support:
- first-run setup and owner bootstrap;
- workspaces, projects, components and environment inventory;
- users, groups, invitations, roles and access reviews;
- module and service installation;
- configuration, secrets references and provider connections;
- repositories, build artifacts, deployments and rollback plans;
- logs, metrics, traces, alerts, backups and restore status;
- audit history and security posture;
- guided troubleshooting and clear explanations for non-developers.

The CLI remains available for automation and advanced work, but an ordinary operator must not need a terminal to perform common administration.

The management console is a platform interface. It is not automatically the public or customer-facing UI of every application.

### C. Installation-owned identity and access platform

Otya should include an optional-but-first-party identity service that each installation operates for itself. It must be useful without an Otya-hosted account or a third-party identity provider.

The installation-owned identity platform should provide:
- a branded sign-in, sign-out, account recovery and invitation experience;
- a local user directory and groups for the operator's own staff or members;
- secure password authentication and password reset;
- passkeys/WebAuthn and authenticator-based MFA;
- session lifecycle, expiry, revocation and device/session management;
- organization and group membership management;
- OIDC/OAuth 2.0 provider endpoints for applications to sign users in;
- signing-key rotation, JWKS publication and audience/scope validation;
- service accounts and workload identity for machine-to-machine access;
- audit records for sign-in, recovery, administrative changes and privilege use;
- optional federation to an external OIDC or SAML identity provider where the operator chooses.

The Engine must use established, security-reviewed protocol and cryptographic implementations. It must not invent its own password hashing, token format, cryptographic primitives or authentication protocol.

Each Engine installation has its own identity issuer, signing keys, user directory, client registrations, sessions and policies. Never use a universal Otya user directory shared by unrelated installations.

### D. Authorization and policy service

Authentication answers "who signed in?". Authorization answers "what may this principal do here?". Keep these separate.

The policy service should support:
- installation-owner and platform-administrator privileges;
- workspace/project/component-scoped roles;
- application-specific roles and permissions;
- groups and resource-level access policies;
- service-to-service capability grants;
- default-deny decisions, least privilege and explicit revocation;
- audit trails for privilege grants and sensitive operations.

A company may use one installation-owned identity realm for staff SSO across its applications. Each application must still define its own resource permissions and business authorization. A successful OIDC login to OPoint must not by itself authorize finance actions. It must not grant access to O's owner-only functions or Omy moderation.

Applications can also choose separate identity clients, groups or realms where stronger isolation is needed. The default is shared staff authentication within one company installation, not shared product authorization.

### E. Shared platform services

The installation can run services once and grant selected projects permission to consume them.

Initial service catalogue:
- persistent storage and migration management;
- file/object storage and media-processing interfaces;
- event bus, scheduler, queues and background workers;
- email, push and notification delivery;
- webhook ingress, signing, replay protection and retry handling;
- secrets vault and credential broker;
- audit service and tamper-evident event export;
- observability: logs, metrics, traces, health and alerts;
- backup scheduling, verification and restore orchestration;
- integration adapters for operator-selected providers;
- optional search, cache and document services.

Shared runtime infrastructure is not shared application data. For example, one storage cluster can host separate project databases and credentials. One mail provider connection can be centrally managed while each project gets a distinct sender configuration, template namespace and scoped credential. A project receives only its granted capability, not administrator access to the shared service.

### F. UI system and application experience kit

The Engine should provide both:
1. a complete management console for non-developers and operators; and
2. reusable UI foundations for developers building end-user applications.

UI foundations can include:
- accessible component library and design tokens;
- responsive layouts, navigation, forms, tables, dialogs and dashboards;
- authentication screens, account settings and profile components;
- role-aware admin views and CRUD patterns;
- localization, theme and branding support;
- starter templates for business apps, portals, media apps and internal tools;
- form and workflow builders for approved, schema-driven use cases.

Do not promise that the Engine automatically creates any complete product UI from a database. Templates and a visual builder can accelerate ordinary screens, while complex application UX remains application-owned. The Engine should not force all products to look or behave identically.

## 3. First-run experience for a non-developer

A new operator should be able to:

1. Install Otya on a computer or server they control.
2. Open a local setup URL or management application.
3. Create the installation owner account and enable MFA.
4. Set the organization name, logo, domain, timezone and recovery contacts.
5. Choose local identity or connect an existing identity provider.
6. Create the first workspace and invite staff.
7. Add projects through a guided wizard (connect a repository, start from a template or register an existing deployment).
8. Choose which built-in services each project needs.
9. Grant roles and service capabilities using clear, human-readable controls.
10. Connect external providers using operator-owned credentials.
11. Create development/staging/production environments and deploy through a preview plan.

The visual console should explain consequences before applying a change. Destructive operations, production releases, credential replacement, database migrations and identity policy changes require explicit confirmation and an audit record.

A developer can perform the same tasks using CLI, SDK, API or version-controlled declarative configuration. The UI is not a separate source of truth; changes should map to versioned, inspectable configuration and management APIs.

## 4. Three experiences, one installation

Do not confuse the platform login page, the operator console and each product's user interface.

- **Identity portal:** sign-in, recovery, MFA and session flows shared by applications that use the installation's issuer.
- **Operator console:** manages the installation, workspaces, projects, services, permissions and deployments.
- **Application UI:** each product's own user experience, which may consume the Engine UI kit and identity service.

These interfaces can share branding and design components while remaining distinct clients with separate permissions and session audiences.

## 5. Isolation and sharing rules

### Across different Engine installations

No automatic sharing:
- no shared identity directory, users, sessions or signing keys;
- no shared databases, files, logs, queues or backups;
- no shared secrets, provider credentials or deployment tokens;
- no shared project registry or control plane;
- no mandatory Otya cloud account, telemetry or remote heartbeat.

An operator may choose an external provider, but that is an explicit connection from their own installation and their own account. Software update downloads and optional license verification are not permission to transmit application data.

Telemetry should be disabled by default or strictly opt-in, with the payload, purpose and destination documented.

### Inside one company's installation

Selected platform services may be shared because the operator owns the trust boundary. Every sharing relationship must be:
- declared and scoped to named projects/environments;
- approved by an installation administrator;
- least-privilege and default-deny;
- independently revocable;
- auditable and visible in the console;
- tested for cross-project isolation.

Production secrets, databases and destructive permissions must never be automatically inherited by development or staging.

## 6. User types and experience

The UI should support different roles without exposing developer complexity to everyone.

| Role | Main experience | Typical permissions |
|---|---|---|
| Installation owner | Setup and security console | Recovery, identity issuer, installation-wide policy and owner administration |
| Organization/platform admin | Visual operations console | Manage members, workspaces, shared services and approved policies |
| Project owner/admin | Project console | Configure one project, its users, data services and releases |
| Developer | IDE, SDK, CLI and project console | Source, tests, builds, development configuration and approved deployments |
| Operator/support | Operations console | Health, logs, alerts, backups and approved recovery tasks |
| Application end user | Product's own UI | Only application-specific entitlements and actions |

An ordinary application user should not see the Engine management interface. An ordinary staff member should not need to understand Workers, D1, Rust, GitHub Actions or provider secrets to perform an approved business task.

## 7. Relationship with Cloudflare and other hosts

Cloudflare is a host/deployment adapter and can also provide optional edge access protection. It is not the Engine's mandatory identity system.

Cloudflare Access demonstrates an identity-aware proxy: it fronts an application, authenticates against configured identity providers and applies access policies before requests reach the origin. Otya should offer its own installation-owned identity portal and policy service so the operator can have a comparable login experience under their control, including when the application runs on Linux or another host.

The Engine may optionally configure a Cloudflare Access application as an additional outer security layer. The application must still validate its own audience, issuer, identity context and authorization. Never put a long-lived Access service token in a browser application or treat the edge gate as a substitute for product authorization.

Host adapters should manage separate workloads on Linux, Cloudflare Workers and future supported providers. A single management console does not imply all workloads run in one process or that Cloudflare-specific login is required for all platforms.

## 8. Relationship with the existing Otya ecosystem

For the first company installation:
- **OEnd** remains the central ecosystem backend/API gateway and product-specific orchestration layer.
- **OID** remains the current identity service and integration boundary during migration. The Engine identity module may eventually host or replace generic identity capabilities only after protocol parity, security testing, data migration and rollback are proven.
- **OPoint** remains the private business application and authority for business, branch, inventory and financial permissions/data.
- **Omy** remains the public media product. Public consumer accounts and local playback behavior stay product-owned. Its use of company staff identity, if any, is a separate and explicitly selected integration.
- **OWeb** remains the public/private website system; its private areas can use the installation's identity issuer.
- **O** remains the private personal assistant with separate owner-only permissions and data.
- **OBackup** remains the current backup/recovery workload until the Engine's backup coordinator and restore tooling have demonstrated parity.

The Engine console should register and manage the existing repositories and Workers as separate projects/components. Do not merge or delete them merely to make the architecture centralised. Move a generic service into the Engine only when the same module can serve an unrelated external application, is versioned and tested independently, and OEnd and product owners approve migration.

## 9. Declarative setup

The long-term .ot format should declare modules, workspaces, identity clients, policies, projects, UI configuration, service grants and environments. It should never contain plaintext secrets.

Illustrative only; not executable in the current repository:

~~~text
installation "otya-company" {
  identity {
    issuer = "https://login.example-company.com"
    provider = "builtin"
    mfa = "required-for-administrators"
  }

  console {
    hostname = "manage.example-company.com"
    branding = "company-brand"
  }

  workspace "company-products" {
    projects = ["oend", "opoint", "omy", "oweb", "o"]

    shared_service "staff-identity" {
      provider = "installation.identity"
      grant = ["oend", "opoint", "oweb"]
    }

    shared_service "mail" {
      provider = "resend"
      grant = ["oend", "opoint", "oweb"]
      credential_scope = "per-project"
    }
  }
}

project "opoint" {
  identity_client = "opoint-web"
  authorization = "application-owned"
  data_isolation = "dedicated-database"
  ui = "business-console-template"
  uses = ["staff-identity", "mail", "audit", "backup"]
}
~~~

The actual language and manifest must use schema versions, strict validation, references to external secret stores, signed module metadata, compatibility constraints, dependency resolution and reviewable plan/apply operations.

## 10. Implementation roadmap

### F1 — Product distribution and operator console
- Decide the complete installable bundle and optional service profiles.
- Create the local supervisor and authenticated management API.
- Build first-run setup and a browser-based operator console.
- Add workspace/project/component/environment inventory.
- Provide role-aware operator workflows without production mutation.

### F2 — Installation-owned identity
- Define the identity issuer and OIDC/OAuth contracts.
- Add local user directory, invitation and recovery flows.
- Add passkeys/WebAuthn and TOTP MFA with safe recovery.
- Implement client registrations, sessions, key rotation and JWKS.
- Add machine identities and service-to-service authentication.
- Add external OIDC/SAML federation as optional adapters.
- Complete threat modelling, security review and independent conformance tests before production use.

### F3 — Authorization and service grants
- Add installation/workspace/project/component roles.
- Add application-specific capability and policy contracts.
- Add default-deny shared-service grants and revocation.
- Add audit and access-review views.
- Prove isolation across projects and environments.

### F4 — Shared service modules
- Add secrets broker, notifications, jobs/events, files, storage, audit and observability.
- Add backup/restore orchestration only after integrity and recovery tests.
- Add provider connectors with per-project scoped credentials.
- Add service catalog, module enable/disable, upgrades and rollback.

### F5 — UI kit and no-code/low-code workflows
- Publish a versioned accessible UI kit and app templates.
- Add schema-driven forms, tables, dashboards and approval workflows.
- Add app setup wizards and visual policy/service assignment.
- Keep source/configuration exportable and reviewable; no hidden proprietary-only state.

### F6 — Deployment and ecosystem adoption
- Manage local Linux services first.
- Add Cloudflare and other host adapters with dry-run plans, scoped credentials, canary, rollback and audit.
- Import current Otya repositories and Workers as read-only inventory first.
- Integrate OEnd and OID as the first carefully bounded pilot.
- Adopt the same released Engine contracts in OPoint, Omy, OWeb and O.
- Never make deployed product runtime depend synchronously on the management console being online.

## 11. Acceptance gates

The platform is not production-ready until tests prove:
- a clean installation operates without an Otya cloud account or remote user directory;
- two independently installed test environments have disjoint users, keys, databases, secrets, logs and service registries;
- an installation can manage multiple applications through one console;
- an application gets only its granted Engine services;
- revoking access blocks subsequent service use and token renewal as documented;
- company staff can sign in once while product-specific resource permissions remain separate;
- public product users do not gain access to internal company applications;
- administrator MFA, account recovery, key rotation and session revocation work;
- the console cannot silently grant production permissions or expose plaintext secrets;
- deployed applications continue when the local management/control plane is unavailable;
- backup restore, upgrade, rollback and uninstall preserve or safely remove operator data as explicitly documented;
- non-developers can complete common member, access, service and operational tasks without using the CLI.

## 12. Current implementation status

The current Otya repository contains an early Rust engine/CLI, configuration and SQLite foundations, an experimental application SDK, and a preview Cloudflare adapter. It does not yet contain the complete operator console, built-in identity provider, production authorization service, full service catalogue, UI kit, visual builder or deployment orchestrator specified here.

This document is a product and engineering target, not a claim that those features have shipped. Do not deploy it as a production authentication system, move existing OID users, replace Cloudflare Access or change production resources based on this specification alone.
