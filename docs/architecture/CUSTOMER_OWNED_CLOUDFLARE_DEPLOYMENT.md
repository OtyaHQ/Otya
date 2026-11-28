# Customer-Owned Cloudflare Deployment Architecture

Status: Proposed architecture decision; implementation not yet complete.

## Decision

Otya Engine is the installable foundation. Opoint and OWeb are applications distributed by Otya and deployed into infrastructure owned by each customer. The default customer deployment must not require a customer GitHub account, Firebase project, or Otya-hosted application runtime.

Each customer connects their own Cloudflare account. Otya Engine provisions and manages the resources required by the selected applications in that account, subject to explicit authorization, account capabilities, product availability, quotas, and plan limits. Customers pay Cloudflare and third-party providers directly where applicable.

## Product boundaries

- Otya Engine: local installer, runtime, application/package manager, Cloudflare provider, deployment planner, provisioning executor, secrets interface, migrations, health checks, audit trail, upgrades and recovery.
- Opoint: customer business application and management UI. Its business data, financial ledger, tenant records, and application runtime live in the customer's deployment.
- OWeb: customer website and web-application builder/management UI. Its projects, assets, configuration and runtime live in the customer's deployment.
- Otya-managed services: optional central services for software releases, signed update metadata, product documentation, optional entitlements, and Otya-operated email/SMS services. These must not be required for core customer operations unless a feature explicitly depends on them.

## Ownership model

The customer owns and controls their Cloudflare account, zones, Workers, D1 databases, R2 buckets, KV namespaces, Queues, Durable Objects, secrets, domains, DNS, logs, backups and billing. Otya software receives only the permissions needed to provision and operate the selected product.

The customer owns their external provider accounts and contracts. Payment gateways, mobile-money services, banking APIs, SMS aggregators and other regulated or commercial APIs require customer-provided credentials and approvals.

Otya must never collect provider passwords or mobile-money PINs. Credentials must be entered through a protected management flow and stored as Worker secrets or another approved secret store, never in source code, browser storage, plaintext configuration, or ordinary application tables.

## Provisioning lifecycle

1. Install Otya Engine and select Opoint, OWeb, or both.
2. Connect the customer's Cloudflare account using an approved OAuth flow or a narrowly scoped API token. Do not request a Global API Key.
3. Inspect account identity, permissions, existing resources, quotas and product availability.
4. Generate a declarative deployment plan from versioned Otya manifests.
5. Display the complete plan, permissions, resource names, expected costs where known, and destructive changes for customer approval.
6. Provision resources idempotently. Re-running must reconcile existing resources rather than create duplicates.
7. Deploy signed application artifacts, apply reviewed database migrations, configure bindings and inject secrets.
8. Run smoke tests, health checks, permission checks and integration checks.
9. Activate the application only after required checks pass; clearly mark optional integrations as unconfigured or degraded.
10. Record an auditable deployment receipt and support rollback of code/configuration where safe.

## Infrastructure provider contract

Cloudflare is the first supported provider, implemented behind a provider interface so other infrastructure providers can be added without coupling application logic to Cloudflare APIs.

The Cloudflare provider should support only the resource types required by the selected application release. Likely building blocks include Workers with Static Assets, D1, R2, KV, Queues, Durable Objects, Workflows, custom domains/DNS and observability. The exact resource graph must be generated from each app manifest, not provisioned wholesale.

MCP may be used as an optional operator/AI interface. It must not be the sole provisioning mechanism. Deterministic, typed provider operations, schema validation, idempotency, retries, state reconciliation and audit logging are mandatory. MCP tool output is untrusted input and must never authorize destructive operations by itself.

## Application data and identity boundaries

Opoint remains authoritative for company membership, branch access, roles, financial records, ledger, inventory and provider transactions. No customer's business data is copied into a central Otya database for routine operation.

OWeb owns its site/project configuration and content within the customer's deployment.

The deployment model must explicitly resolve identity. A customer may use a local Opoint identity mode or an optional Otya ID federation mode if the product requires shared identity. Core customer operations must not silently depend on an unavailable Otya central service. Existing OID integration must be reviewed and migrated deliberately; it must not be removed by assumption.

## Communications

Separate platform communications from customer business communications.

- Otya may provide centrally managed product-release notices, account/security notices and optional email delivery as a managed service.
- Customer business emails may use an Otya-managed relay or a customer-owned provider, with sender/domain verification and clear delivery ownership.
- Transactional SMS, OTP and business alerts should use customer-selected providers (for example, an approved EgoSMS account) and customer-supplied credentials.
- Push notifications must not require a customer Firebase project by default. Evaluate Web Push and platform-native push delivery separately; native mobile push may still require platform vendor credentials or an Otya-operated delivery service.
- A notification is never proof of payment or transaction completion. The application must read authoritative transaction state.

## Security requirements

- OAuth authorization code with PKCE where supported; otherwise narrowly scoped, customer-created API tokens.
- Least privilege, explicit account selection, token encryption at rest, rotation, revocation and no frontend exposure.
- Separate development, staging and production resources and credentials.
- Customer approval before destructive actions, domain changes, production migrations or resource deletion.
- Signed releases, artifact provenance, dependency scanning and reproducible build metadata.
- Tenant isolation and server-side authorization on every business-data request.
- Redacted logs; never log secrets, PINs, OTPs or full sensitive provider payloads.
- Backup/restore procedures and documented data export/deletion.
- Complete audit events for authorization, provisioning, secret changes, deployment, migration and recovery.

## Operational requirements

The engine must support plan/apply/diff, dry-run, resumable provisioning, idempotent retries, drift detection, resource inventory, health checks, deployment history, safe upgrades, migration status, backups, export, uninstall and recovery. Uninstall must default to preserving customer data and require explicit confirmation for deletion.

Cloudflare account limits, paid features, regional availability, pricing and API permissions must be checked at runtime and documented per release. Otya must not promise zero hosting cost or universal availability.

## Migration from current ecosystem assumptions

Audit Opoint's existing assumptions about the shared OEnd/API gateway, OID, Firebase, Resend and Cloudflare resources. Classify each capability as customer-local and deployed into the customer's account; an Otya-managed optional service; a customer-selected third-party integration; or a platform capability that is not required for this deployment profile.

Do not silently break existing Otya-operated deployments. Introduce a versioned customer-owned deployment profile and migrate through explicit compatibility stages.

## Acceptance criteria

A release is not complete until a clean customer Cloudflare account can be connected without GitHub or Firebase setup; the engine can produce and apply a reviewed plan; Opoint and OWeb deploy into that account; required resources and bindings are verified; customer secrets can be added without exposing them; provider sandbox tests pass; updates are idempotent; revoke/disconnect works; customer data remains exportable; and the core application continues operating when Otya's optional central services are unavailable.

## Open implementation decisions

1. Whether the first delivery target is a local CLI/desktop installer, a browser-based management console backed by a local agent, or both.
2. Whether Cloudflare connection uses Cloudflare OAuth, customer-created scoped API tokens, or both depending on deployment environment.
3. Whether customer-owned Opoint deployments support local identity first, with Otya ID federation as an optional mode.
4. Which email and push capabilities Otya operates centrally versus which customers configure themselves.
5. The exact Cloudflare resource graph and minimum plan for each Opoint and OWeb feature.
