# Otya Engine Modules and Adapters

## Separate business logic from provider mechanics

A business module owns domain meaning, authorization and invariants. An adapter translates a stable engine contract to an external system. The engine core owns lifecycle, permissions, common reliability, diagnostics and version negotiation.

Do not put provider-specific SDK calls throughout business modules.

## Module classes

1. **Core modules:** lifecycle, configuration, authorization hooks, audit, operation state and health.
2. **Business modules:** accounting, organization/staff, inventory, retail, commerce, communications and other separately versioned domains.
3. **Infrastructure adapters:** SQLite/PostgreSQL, local filesystem, object storage, secret stores and network transports.
4. **Provider adapters:** Cloudflare, GitHub, Resend, Firebase, SMTP, generic REST and signed webhooks.
5. **Host bindings:** C ABI, Flutter/Dart and Python wrappers.

## Module contract

Every module declares:
- stable module ID and semantic version;
- compatible engine API range;
- dependencies and optional features;
- capabilities requested and why;
- configuration schema and secret references;
- storage/schema migrations it owns;
- health and readiness checks;
- lifecycle hooks and graceful-shutdown behavior;
- audit event types and data classification;
- tests and known limitations.

The module registry must reject duplicate IDs, incompatible versions, undeclared capabilities and missing migrations. Disabling a module must not silently drop its customer data.

## In-process versus isolated modules

First-party trusted modules may compile as Rust crates against a versioned engine API.

Do not load arbitrary native shared libraries into the privileged engine process. They share its address space and can bypass process-level isolation.

Third-party or untrusted extensions should initially run as separate processes behind a narrow authenticated local protocol. A sandboxed WebAssembly Component Model may be evaluated as a later ABI. This does not create or require an Otya language; extension authors can use supported external toolchains.

## Adapter conformance

Each adapter must publish:
- supported provider/API versions and regions;
- auth method and minimum scopes;
- credential source and rotation behavior;
- outbound/inbound data fields;
- request timeout, rate limit and retry contract;
- idempotency and reconciliation capabilities;
- webhook signature and replay verification;
- health check and diagnostic detail;
- error mapping and operator remediation;
- test/sandbox mode and test fixtures.

The engine exposes a provider-neutral result; it must not erase provider uncertainty or invent success.

## Initial provider mapping

### Cloudflare

Optional customer-owned edge and infrastructure adapter. It may deploy a customer-authorized Worker or use selected Cloudflare services. It is not required for local engine operation. Never store broad Cloudflare account tokens in ordinary module config.

### GitHub

Optional source/release adapter for repositories, releases and approved Actions workflows. Prefer GitHub App installation permissions or OIDC exchange where appropriate. It is not a runtime dependency of an installed customer engine.

### Resend

Optional outbound email adapter. It is transport only: the business module decides whether a message is authorized, the recipient and category. Verify webhook signatures against the exact raw body, deduplicate event IDs and preserve provider delivery state. Provide SMTP or another provider alternative.

### Firebase

Optional mobile messaging and selected app-integrity/identity adapters. Push notifications are hints; the owning application API remains authoritative. Firebase credentials are customer-controlled. Firebase is not a required engine database or identity authority.

### Generic adapters

A stable HTTP client policy, SMTP, signed webhook, local file and S3-compatible interfaces reduce provider lock-in. Provider SDKs remain optional dependencies.

## Testing

Use local fake providers, recorded sanitized fixtures and provider sandboxes. No adapter test should require production credentials. Live smoke tests must be a separate, explicit operator action and must identify exactly what will be sent or changed.
