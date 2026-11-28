# Bring-your-own-infrastructure provider architecture

**Status:** target architecture and security specification. The Rust core now has initial metadata contracts for provider connections, credential references, resource bindings and project grants. Formatting, Clippy, workspace tests, release build and the Rust 1.85 minimum-version check passed in Otya Engine CI run 46159074134. The portable Android and WebAssembly core targets also passed; the separate Cloudflare Worker packaging check remains in progress. The registry, vault and provider adapters are not yet shipped.  
**Reviewed:** 2026-11-28.

## 1. Product rule

Otya is not a hosting company and does not supply mandatory cloud resources.

The operator installs Otya in an environment they control. The operator chooses whether to run workloads locally, on a VPS, in their own Cloudflare account, in their own Firebase/Supabase projects, or with another provider. The operator creates and pays for those external accounts directly.

Otya supplies the runtime, SDK, CLI, application contracts, local visual console, guided setup, provider-neutral service interfaces, provider-specific adapters, secure credential handling, deployment orchestration, diagnostics, reusable UI, identity and application modules.

The operator supplies provider accounts and subscriptions; projects, databases, buckets, Workers and other cloud resources; domains, DNS, TLS and public endpoints; API credentials, OAuth grants, webhook secrets and service-account credentials; data, policies, consent and decisions about service use.

Otya never silently provisions paid resources, creates a provider account, selects a paid plan or changes production infrastructure. Provider mutations require an explicit plan, permission check and approval.

## 2. Provider-neutral architecture

A connector is an adapter to a service the operator already owns. A connector is not a copy of that provider, and it must not claim that all provider products are interchangeable.

~~~text
Operator's Otya installation
          |
          +-- Otya Console
          |     +-- Provider catalogue
          |     +-- Add connection wizard
          |     +-- API / webhook setup
          |     +-- Secret and scope manager
          |     +-- Test / health / usage view
          |
          +-- Local credential vault
          |
          +-- Provider connection registry
          |     +-- Cloudflare account connection
          |     +-- Firebase project connection
          |     +-- Supabase project connection
          |     +-- SMTP / Resend / other mail
          |     +-- Payment / AI / analytics / storage
          |     +-- Custom REST / OAuth / webhook
          |
          +-- Versioned Otya capability contracts
                +-- relational database
                +-- object/file storage
                +-- identity/authentication
                +-- messaging/notifications
                +-- jobs/events/webhooks
                +-- AI/model inference
                +-- deployment/edge runtime
                +-- payment and other domain APIs
~~~

Each provider adapter advertises the exact capabilities, operations, limits, authentication modes and environments it supports. A connection to Supabase does not imply access to Cloudflare R2, and a Firebase project is not silently treated as a generic SQL database.

Applications depend on Otya interfaces where that improves portability. If an application relies on a provider-specific feature, the dependency is declared explicitly in its manifest and documentation.

## 3. The operator connection workflow

The ordinary workflow should be available in Otya Console without requiring terminal commands.

1. Select **Connections → Add provider**.
2. Choose a provider and the capability needed (for example Cloudflare deployment, Supabase database, Firebase messaging, SMTP email or an AI API).
3. Otya explains the exact provider account, resource and minimum permission needed. It links to the correct provider dashboard/documentation.
4. The operator authorizes using OAuth where supported, or creates a narrowly scoped API token/service credential and enters it into the local Otya Console.
5. Otya stores the credential in the local vault and validates it using non-destructive read-only checks.
6. The operator selects which workspace, projects and environments may use the connection.
7. Otya shows a capability map, permission check, connection health and any missing setup.
8. The operator runs a sandbox/test action.
9. Any create/update/delete/deploy action produces a preview plan showing exact resources and effects, then asks for approval.
10. The operator can rotate, revoke, disable or delete the connection. The console shows which projects and deployments depend on it.

A provider connection and a resource binding are different:
- **Connection:** permission to use a provider account/API.
- **Resource binding:** selected resource inside that account, such as one D1 database, R2 bucket, Firebase project, Supabase project or SMTP sender.
- **Project grant:** which application/environment may use that binding.
- **Webhook endpoint:** a receiving URL and verification policy used by a provider to send events into an application.

The console must not treat "connected" as proof that every service, permission or resource is configured correctly.

## 4. Secrets and credential protection

Credentials entered into Otya Console must be sent only to the operator's own installation over TLS. Never ask the operator to paste provider secrets into public Otya chat, a public issue, source code or an ordinary application form.

The platform must provide:
- a local vault interface backed by an OS secret store, an operator-managed vault/KMS, or encrypted local storage with an operator-controlled master key;
- envelope encryption and key rotation for stored credentials;
- project/environment-specific secret namespaces and default-deny grants;
- write-only secret entry: after saving, reveal no plaintext value in the UI or API;
- redacted logs, diagnostics, audit events and error messages;
- no secrets in manifests, Git, browser local storage, frontend bundles, query strings or URLs;
- scoped provider identities, expiry/rotation metadata and last-validated status;
- explicit export policy. Raw secrets are excluded from ordinary workspace export and backups by default; recovery procedures are documented;
- a clear warning that a compromised operator host or administrator can compromise secrets accessible to that installation.

Separate credentials by purpose: local Otya provider-management credential used to deploy/manage resources; runtime credential consumed by the deployed application; public configuration safe to publish; webhook signing secrets used to authenticate inbound events.

For remote deployments, Otya should pass runtime secrets into the destination's native secret manager and bind them to the selected workload. Keep the provider management token in the operator's local vault; do not bundle it into application code or ship it to the Worker/container. The platform must show where each secret is stored and which component can use it.

Secrets must never be returned to a browser after initial write. On rotation, write a new version, prove the dependent workload has accepted it, then revoke the old credential according to a reviewed plan.

## 5. API and webhook onboarding

### External APIs

A connector wizard should accept OAuth authorization (preferred when supported), an API token with documented scopes, legacy username/password only where no safer protocol exists, mTLS credentials managed by the vault, or a custom HTTP endpoint with declared authentication and optional OpenAPI document.

For a custom API the operator supplies the base URL, environment, authentication mode, required scopes, timeout/rate-limit expectations and (where available) OpenAPI specification. Otya validates the connection with a safe read-only request. It does not blindly execute imported API descriptions or arbitrary webhook instructions.

Each connector documents outbound fields, data processing, region, quotas, rate limits, retries, idempotency, error mapping, sandbox support and provider-specific failure modes.

### Webhooks

Otya can generate an application-specific receiving endpoint and signing secret for an application hosted by the operator. The operator registers that URL in their provider dashboard or through an explicitly authorized provider API.

The receiving endpoint must verify the provider's official signature over the raw request body; validate timestamps and reject replays; deduplicate provider event IDs and handle retries idempotently; enforce request-size, content-type and rate limits; acknowledge only after safely persisting or queuing the event; redact personal and secret data from logs; and support secret rotation and test events.

Otya does not make a private local machine publicly reachable by itself. If a provider requires a public webhook URL, the operator must supply a public endpoint through their chosen host, domain, tunnel or deployment. Otya can configure it through an approved provider adapter, but it does not host the endpoint on Otya infrastructure.

## 6. Provider-specific integration patterns

### Cloudflare

The operator creates and owns the Cloudflare account, zone, Workers, D1 databases, KV namespaces, R2 buckets, Queues, Workflows, DNS and billing.

Otya's Cloudflare adapter can validate an account connection and token scopes; inventory resources the token is allowed to see; create a deployment plan and deploy Workers; configure bindings, environment variables and Worker secrets; bind the application to the operator's supported resources; configure routes and custom domains only with explicit permission and approval; and inspect deployment versions and health.

Use a Cloudflare API token with the minimum product/resource scopes for the intended operation. Do not ask for the operator's global API key or an account-wide token when narrower permissions suffice. Distinguish read-only inventory from Worker deployment, DNS and resource-creation permissions.

When code is already running inside a Worker, use native Cloudflare bindings for supported resources such as D1, R2, KV, Queues and Workflows rather than making unnecessary REST API calls. The Cloudflare host adapter should use Worker bindings; the external management adapter can use scoped Cloudflare APIs. Keep those two roles separate.

Sensitive runtime values must use Worker Secrets or an appropriate Secrets Store binding, not plaintext environment variables or source-controlled configuration. The operator's Cloudflare account remains responsible for resource availability, usage, limits and billing.

### Firebase

The operator owns the Firebase/Google Cloud project and configures the Firebase services required by the application (for example Authentication, Firestore, Realtime Database, Cloud Storage, App Check or messaging).

The connector distinguishes public Firebase application configuration from privileged server credentials. Firebase's normal Firebase API key identifies a project/app and is public by design; it is not an authorization boundary. Secure data access with Firebase Security Rules and App Check, and restrict keys to the appropriate APIs. Service-account private keys and other privileged Google Cloud credentials are secrets and must remain server-side in the local vault or destination secret manager.

The adapter exposes only the capabilities enabled and permitted in that specific Firebase project. A Firebase API-key check does not prove that Security Rules, Auth providers or App Check are configured safely.

### Supabase

The operator owns the Supabase project and its billing. The connector accepts the project URL and the correct key types for the code location.

Use a publishable key in public/browser/mobile clients only with appropriate Postgres grants and Row Level Security (RLS) policies. Use a secret key only inside trusted backend components; secret keys bypass RLS and must never be put in browsers, mobile bundles, public source or end-user configuration.

Prefer current sb_publishable and sb_secret keys. Legacy anon and service_role keys are still encountered in existing projects, but Supabase documentation says they are being deprecated in early 2027. Provide a migration helper, but never rotate or revoke the old key until dependent clients are migrated and tested.

A successful API-key check is not proof that RLS is correctly configured. Otya should provide an RLS audit checklist and safe diagnostics, not falsely certify a user's policies.

### Email, payments, AI and other APIs

The provider catalogue supports first-party maintained adapters and generic standards-based connectors:
- SMTP or transactional email providers;
- payment gateways and their webhook/reconciliation APIs;
- AI/model endpoints with model-specific capabilities and limits;
- storage and CDN APIs;
- analytics, search and messaging systems;
- custom REST/OpenAPI and signed webhook integrations.

Payments require idempotency, trusted amount/currency calculation, verified webhooks and reconciliation. AI connectors need provider/model identification, usage/price limits controlled by the operator, data-handling disclosure, timeouts and fallback policies. An API adapter does not make a provider legally compliant or guarantee a successful transaction.

## 7. Local, external and hybrid operating modes

Otya should not force a cloud provider.

- **Local-first:** use local Linux services, SQLite/PostgreSQL, local files and locally supervised jobs. External APIs are optional.
- **Bring-your-own-cloud:** deploy to accounts and resources the operator owns, such as Cloudflare, Supabase or Firebase.
- **Hybrid:** run the Otya control plane on the operator's Linux host, while applications deploy to the operator's cloud accounts. The control plane manages releases and health, but applications continue running if the local management console goes offline.

The control plane is not in the request path for every application request. Already deployed workloads should not stop merely because the operator's laptop or local console is offline.

Otya may build, package and upload application artifacts with operator approval. It does not supply compute or storage; the operator pays the hosting provider directly. If the operator has not connected a provider or configured local service, features needing that dependency remain unavailable and the console explains what is missing.

## 8. Execution and performance model

Otya should not put its management server in the hot path of every application request.

The management/control plane configures projects, validates provider accounts, prepares resources, assigns grants, injects runtime secrets and manages releases. After deployment, application requests should run in the application's selected host/runtime and use the appropriate local Otya module, native provider binding or direct provider client.

Examples:
- A Cloudflare Worker uses its own D1/R2/KV/Queues bindings when those resources are part of the same Worker account/runtime. Do not route those operations through the Otya Console or call Cloudflare's REST API unnecessarily from inside the Worker.
- A browser may use a provider's public client key only when the provider's documented security controls make that safe (for example Supabase publishable key plus correctly configured RLS). Privileged secrets remain in backend runtimes.
- A Linux application may use the local Otya service over authenticated IPC or embed the library. A local database/file operation should not make an unnecessary remote round trip.
- A workload deployed to a user's account should continue operating if the Otya management console, laptop or local controller is offline.

Otya's performance is composed of the portable Rust core, host/runtime overhead, adapter overhead and external provider latency. Otya can optimize its own code, batching, connection reuse, concurrency, serialization and retries; it cannot guarantee the uptime, response time, quotas or geographic latency of a provider the operator chose.

The platform should measure and expose:
- local Engine execution time separately from external provider time;
- queue wait, retry and provider throttling separately from handler latency;
- adapter and serialization overhead;
- request/operation IDs across service boundaries;
- timeouts, cancellation and bounded concurrency;
- cost/usage only when the provider exposes reliable usage data and the operator grants read scope.

Never hide provider latency inside one generic "Otya response time". Never retry non-idempotent operations without an idempotency/reconciliation contract. A provider outage should not stop unrelated local modules or applications.

## 9. Build any application or system

The Engine is not a Rust-only app framework. Rust is the preferred implementation language for the core and native embedded SDK, but external developers should be able to build applications in any language that can call a supported, documented interface.

Target developer interfaces:
- Rust embedded SDK for in-process/native applications;
- versioned HTTPS/JSON management and runtime APIs with OpenAPI specifications for TypeScript, Python, Go, Java, .NET, PHP, Ruby and other ecosystems;
- authenticated local IPC for applications running beside an Otya service;
- stable C ABI for languages that need native embedding;
- optional language SDKs generated from the same versioned contracts;
- optional .ot manifests/configuration for declarative project and service wiring, without making .ot mandatory for app source code.

A developer keeps source code in their own repository and controls the frontend, domain model, product authorization and deployment strategy. Otya provides optional runtime contracts and services, provider adapters, identity primitives, UI components, templates, test harnesses and management tools.

An application must not require an Otya-hosted account, Otya repository, Otya runtime host or connection to another operator's installation. It can consume released SDK packages or documented APIs from its own installation.

The repository must publish an explicit supported-use license and contribution terms before claiming unrestricted third-party use, redistribution or contributions. The license decision is still open; repository visibility alone is not a grant of reuse rights.

## 8. Capability catalogue and honest readiness

Every service entry must show whether it is built into Otya or requires an external provider; local adapter availability and each external provider adapter; authentication method and required scopes; resource types and supported operations; whether an operation is read-only, mutating, destructive or billable; development/test/production support; limitations, provider quotas and regional constraints; required credentials and where each will be stored; health and last-verified state; and fallback behavior or an explicit statement that no fallback exists.

A provider adapter must never report success unless the provider confirms it. When an API is unavailable, credentials expire or quotas are exceeded, return a typed error and actionable operator guidance. Do not silently switch providers if doing so changes data location, cost, security or legal terms.

## 9. Branding and provider independence

Otya owns the platform experience and names its own abstractions. Connected providers remain clearly attributed as external services. The console shows the provider name, account/project reference, capabilities granted, credential location, billing owner and health status.

Brand Otya as the independent engine/control plane, not as a cloud reseller or an invisible copy of another dashboard. The operator retains ownership of provider accounts, domains, billing, usage and data.

## 10. Acceptance tests

- No provider account is created and no paid resource is provisioned during installation.
- The platform works offline/local-only when no external connections are configured.
- Two separate Otya installations do not share provider connections, secrets or resource inventory.
- A provider can be connected to development without granting production access.
- The UI never displays saved plaintext secrets or places them in logs, manifests, builds or browser storage.
- The connector reports insufficient scopes and missing services distinctly.
- Cloudflare tokens are scoped to the intended account/products/resources; deployments use provider-native bindings and secrets.
- Firebase public API key versus service-account credential handling is correctly classified.
- Supabase secret keys are never exposed to frontend artifacts, and the tool checks RLS configuration separately from key connectivity.
- Webhooks validate signatures, timestamps, event IDs, idempotency and body limits.
- Read-only tests cannot mutate production; mutating operations require preview and approval.
- A disconnected provider does not take down unrelated projects or local Engine functionality.
- Existing deployed applications continue while the operator control plane is unavailable.

## 11. Current implementation status

The existing Engine now has initial Rust metadata types for ProviderConnection, CredentialReference, ProviderResourceBinding and ProviderProjectGrant, including basic validation for identifiers, duplicate scopes, missing credential references, resource/project/environment matching and capability grants. These types hold secret references, not raw secret values. Formatting, Clippy, workspace tests, release build and Rust 1.85 checks passed on the provider-contract revision (Otya Engine CI run 46159074134). Android and WebAssembly portable-core compilation also passed; the separate Cloudflare Worker packaging job remains in progress.

The Engine still has no persistent provider connection registry, credential vault or resolver, guided integration console, OAuth flow, generic webhook receiver, live provider API adapter or production provider deployment manager. The current Cloudflare Worker adapter remains a preview/demo. The provider patterns in this document are target design, not shipped integrations.

Do not paste live provider credentials into the repository, public issue, logs or this documentation. No production resources or credentials are changed by this specification.
