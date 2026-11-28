# AI application creation and infrastructure provisioning

**Status:** target architecture and implementation plan; the guided builder, MCP client, application generator and provisioning workflow are not yet shipped.
**Reviewed:** 2026-11-28.

## 1. Product definition

Otya is an AI-assisted application creation and operation platform installed and controlled by each individual or organization. A non-developer describes the business, website, application or internal system they need. Otya turns that intent into a reviewable application plan, composes reusable Otya modules, provisions resources in provider accounts owned by the operator, generates the application and deploys it to infrastructure selected by the operator.

Otya does not host customer applications, databases, files, domains, AI inference or provider accounts. The operator supplies and pays for infrastructure and authorizes Otya to manage it. Otya's public website distributes documentation, installers and releases; it is not the runtime or control plane for customer projects.

The core workflow is:

1. Install or open the operator's own Otya Console.
2. Create the local installation owner and identity realm.
3. Connect a provider account (Cloudflare is the recommended first guided path).
4. Describe the desired application in ordinary language.
5. Let Otya analyze requirements and ask only material follow-up questions.
6. Produce an architecture, module list, data model, provider resource plan, permissions and cost/limit notes.
7. Validate the plan and show every intended mutation.
8. Obtain explicit approval for resource creation, paid actions, public exposure, secrets and production deployment.
9. Generate source, schema migrations, UI, API contracts, tests, deployment manifests and documentation.
10. Provision approved resources, deploy to a preview, run acceptance checks and show the operator the result.
11. Promote to production only after a separate explicit approval.
12. Keep the project editable, exportable and operable without an Otya-hosted service.

## 2. Cloudflare-first, not Cloudflare-only

Cloudflare is the first-class reference deployment provider because it offers Workers, D1, R2, KV, Durable Objects, Queues, Workflows, DNS, Access and other capabilities, and it exposes an official remote MCP server at `https://mcp.cloudflare.com/mcp`. That MCP server uses OAuth authorization and can inspect and change resources within the permissions the operator grants.

Otya should implement a standards-based MCP client and provider adapter, not depend on a ChatGPT-specific connector. The Cloudflare connection flow should:
- open the provider's OAuth authorization page;
- display the account identity and granted scopes;
- bind the connection to the operator installation;
- perform read-only account/resource discovery first;
- keep access tokens in the operator's secret store;
- expose only approved tools to the planning/execution agent;
- log tool calls and their project/environment scope;
- support disconnect, token revocation and scope review.

Where an official MCP server is unavailable, use the provider's supported OAuth/API, CLI or SDK with the same Otya capability contract. Support Firebase, Supabase, GitHub, email, payment, AI and generic REST/OpenAPI/webhook providers as separate adapters.

Cloudflare is a recommended first provider, not a universal database requirement. The builder must choose storage based on application requirements and the operator's choice. D1 is appropriate for many Worker-native relational workloads; R2 for objects/files; KV for globally distributed key-value/configuration; Durable Objects for coordinated state and realtime; Queues/Workflows for asynchronous work; and external PostgreSQL/Supabase or other databases where their semantics, ecosystem or scale are more suitable. Provider limits, pricing, regional/data-residency constraints and service availability must be surfaced before approval.

## 3. Application blueprints and composition

An application request becomes a versioned, provider-neutral blueprint, not an unstructured prompt directly executed against production.

A blueprint records:
- application identity, purpose, audience and owner;
- features and user journeys;
- roles, permissions and data ownership;
- domain entities, relationships, retention and migration plan;
- UI pages, navigation, forms and design tokens;
- APIs, events, webhooks and external integrations;
- required Otya modules and versions;
- provider capabilities and exact resources;
- environments, domains, deployment components and runtime;
- secrets references (never secret values);
- security, privacy, backup and recovery requirements;
- acceptance tests, observability and operational runbooks.

Blueprint generation is followed by deterministic validation. The AI can propose a blueprint and explain trade-offs; a compiler/planner validates schemas, capability compatibility, permission scope, resource naming, provider quotas, secret references and migration safety before any execution.

The generated project must include normal editable source code. It must not be a locked hosted artifact or require the AI to be online for the application to run.

## 4. Reusable modules and the existing Otya products

Existing Otya products are the first reference implementations and extraction sources, not mandatory dependencies for every generated application.

Potential reusable capabilities to extract behind stable contracts:
- OID: identity protocol, session lifecycle, account recovery and access primitives;
- OEnd: API routing, service orchestration, integration patterns and shared backend contracts;
- OPoint: business administration patterns, staff workflows, audit and operational modules;
- Omy: media catalog, playback integration, asset delivery and consumer-facing media patterns;
- O: assistant interaction patterns, tool permissions and AI workflow orchestration;
- OWeb: website layouts, public/private content patterns and reusable UI components;
- OBackup: backup manifests, verification, restore workflows and operational reporting.

Extraction requires removal of product-specific business rules, secrets, user data, proprietary configuration and assumptions. Each module gets a documented API, schema/migrations, permissions, tests, compatibility policy and explicit provider dependencies.

OEnd remains the central backend for the Otya company's own ecosystem. It is not a mandatory runtime dependency for unrelated applications generated by Otya. A generated project may opt into an OEnd service only when the operator explicitly chooses that integration.

The first conversion targets should be templates and reusable modules, not a rewrite or merger of the current products. Keep existing repositories and production Workers intact until parity, migration, rollback and data ownership are proven.

## 5. Provisioning and deployment plan

The planner should emit a declarative resource graph and a human-readable change plan. For example, an online store might request:
- one Worker for the application API;
- one static frontend deployment;
- one D1 database with versioned migrations, or a selected external SQL provider;
- one R2 bucket for product images and uploads;
- a Queue for order emails or background fulfilment;
- an email provider connection;
- Otya Identity or an explicitly selected external identity provider;
- DNS/custom domain and TLS configuration if requested;
- scoped runtime secrets and bindings.

The exact graph depends on requirements and selected providers. Do not create every possible resource by default.

Provisioning must be idempotent and stateful:
- discover existing resources before creating;
- use stable project/environment/resource identifiers;
- produce a dry-run plan and expected cost/quotas where available;
- require approval for create/update/delete, billing changes, public routes and production;
- record provider resource IDs and ownership tags/metadata;
- handle partial failure with resumable operations and compensating actions;
- never delete a resource merely because it is absent from a generated plan;
- detect drift and show a proposed reconciliation;
- preserve exportable infrastructure manifests and application source.

Use provider-native bindings for runtime-to-resource access where available. Do not route every runtime request through the Otya control plane. The management plane provisions and observes; deployed applications call their provider bindings or configured services directly.

## 6. AI agent and MCP safety model

The AI is a planner and operator assistant, not an unrestricted administrator.

Use distinct execution phases:
- **Discover:** read-only inventory and documentation lookup.
- **Plan:** produce a blueprint and resource graph without mutations.
- **Validate:** deterministic schema, policy, capability, quota and security checks.
- **Approve:** show exact changes and obtain an explicit human decision.
- **Execute:** perform only approved, scoped operations.
- **Verify:** test deployment and resources; compare actual state with the plan.
- **Operate:** monitor, diagnose and propose changes; production mutations require approval according to policy.

Tool security requirements:
- OAuth 2.1/MCP authorization where supported; no copying provider passwords into prompts.
- Least-privilege scopes and separate read-only versus mutation grants.
- Installation, workspace, project and environment scoping on every tool call.
- Tool allowlists and argument validation; no arbitrary shell or unrestricted API execution from model text.
- Treat provider output, repository contents, web pages, issues and application data as untrusted input, never as instructions to the agent.
- Secret redaction in model context, logs, traces and generated files.
- Explicit approval for destructive actions, cost-incurring resources, production deploys, DNS changes, access policy changes and secret rotation.
- Audit actor, tool, target resource, plan ID, approval and result.
- Rate limits, operation budgets, timeouts, cancellation and rollback/recovery paths.

MCP is a protocol for tool discovery and execution; it does not itself make an integration safe. Otya must implement authorization, policy enforcement, validation and audit around every server and tool.

## 7. User experience: developer and non-developer

Offer two views over the same project and blueprint:
- **Guided mode:** describe an outcome, review visual architecture, approve setup, edit content and workflows, inspect health and publish.
- **Developer mode:** inspect and edit source, APIs, schemas, migrations, manifests, logs, tests, provider bindings and deployment plans.

Do not create a separate low-code platform with a different runtime. Both modes should generate and operate the same normal application artifacts.

The Console should clearly distinguish:
- Otya-managed local control plane;
- operator-owned provider accounts and billing;
- generated project source and ownership;
- provider resources created for each project/environment;
- optional Otya modules versus provider-specific dependencies;
- preview versus production state.

## 8. No-hosting and ownership invariants

- No Otya-hosted database, runtime, file bucket, user directory or mandatory AI endpoint.
- Each operator installation has independent users, signing keys, configuration, vault, projects and provider accounts.
- The operator owns repositories, source, data, domains, provider accounts and billing.
- Applications continue running if the local Otya Console is offline, subject to their own provider/runtime availability.
- The operator can export source, blueprint, schema, infrastructure manifests and operational records.
- Otya does not silently create provider accounts, subscribe to paid plans or move data.
- Otya may optionally use local AI or an AI provider account supplied by the operator; AI use must not be mandatory for the generated application to run.

## 9. Delivery sequence and acceptance gates

### A. Provider connection foundation
- Persistent provider registry and installation-local encrypted secret store.
- Cloudflare OAuth/MCP connection with read-only discovery.
- Resource inventory, capability catalogue and account identity verification.
- Acceptance: connect a disposable Cloudflare account, enumerate allowed resources, disconnect and prove credentials are not logged or exposed.

### B. Declarative application blueprint
- Versioned blueprint schema and `.ot` authoring format.
- Project manifest, capability graph, environment and resource graph.
- Deterministic validation and JSON output.
- Acceptance: a sample store blueprint validates without calling an AI or provider.

### C. Safe provisioning engine
- Plan/diff, approvals, idempotency, durable operation log and resumability.
- Cloudflare adapter for Workers, D1, R2, KV, Queues, Durable Objects, Workflows, DNS and secrets/bindings as supported.
- Acceptance: create an isolated preview stack, verify it, rerun without duplicates, and clean up only explicitly owned test resources.

### D. AI application builder
- Chat-based requirements gathering and blueprint generation.
- Tool selection constrained by provider capabilities and approval state.
- Source, schema, UI, tests and documentation generation.
- Acceptance: create and edit a sample catalog/store app, deploy preview, run tests, export source, and continue operating it without Otya cloud services.

### E. Reusable Otya application modules
- Extract generic capabilities from existing products behind versioned contracts.
- Build reference templates for business management, media, websites and assistant workflows.
- Acceptance: each reference app builds from modules without importing private production data or requiring OEnd unless explicitly selected.

### F. Visual application studio and ecosystem migration
- Guided UI/forms/workflows and developer source mode.
- Migrate selected Otya applications one module at a time after parity and rollback testing.
- Acceptance: a non-developer can configure and operate a supported app; a developer can take over the same project and source.

## 10. Current implementation boundary

This document defines target architecture. It does not claim the following are implemented: MCP client, Cloudflare OAuth, provider registry, encrypted vault, resource planner, deployment orchestrator, application blueprint compiler, AI builder, visual Studio, or extraction of existing product services. Existing provider metadata contracts are only an initial Rust model. No provider accounts, production resources, Workers, databases, credentials or deployments are changed by this architecture document.
