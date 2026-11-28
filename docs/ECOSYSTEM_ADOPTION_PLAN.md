# Otya ecosystem adoption plan

**Status:** first integration pilots prepared; production adoption is not complete.  
**Reviewed:** 2026-11-28.

Otya Engine should power shared runtime capabilities underneath Otya applications and websites. It must not absorb product ownership, user interfaces, data authority, authentication or legal/business rules.

## Product boundary

- **Otya Engine** is the reusable headless foundation: runtime contracts, application SDK, host adapters, storage/operation primitives and later durable jobs, events and provider adapters.
- **OEnd** remains the shared backend/API gateway and integration owner. It is the first integration host because it already fronts product services and can call a private Engine service binding.
- **OPoint** remains the authority for company membership, branches, retail, inventory and financial records. It is private internal company software, not a public SaaS offering.
- **Omy Play** remains the public consumer media product. Playback and local media must remain useful offline; Otya Engine must not become a prerequisite for local playback.
- **O** remains the private owner-facing application. Privileged operations, approvals and owner data stay protected by OEnd/OID authorization.
- **OWeb** is the intended public website/browser frontend. Its GitHub repository was empty at this review, so it is a future application surface, not yet an Engine adoption target.
- **OHub** is public developer/partner documentation, not the OWeb product frontend and not a runtime application.
- **O** and **Omy Play** have Flutter clients. Their Android packages are clients of product APIs; they do not turn Otya Engine into an APK or Android app.

## Repository consolidation decisions

Consolidate shared contracts and infrastructure; do not merge product repositories simply to make them appear to use one engine.

| Keep separate | Combine or standardize |
|---|---|
| Otya Engine, OEnd, OID and each product's source/data authority | Shared versioned API contracts, request/error conventions, service-binding rules and CI templates |
| OPoint business rules, company/branch access and financial ledger | Generic idempotency, retry, scheduling and audit primitives only after Engine conformance and product acceptance tests |
| Omy Play client, local media, playback, private vault and product backend | Connected-service plumbing where it does not weaken offline behavior |
| O owner application, privileged approvals and owner data | Reusable non-privileged task execution after OEnd/OID authorization |
| OWeb public website and OHub developer documentation | Shared brand, navigation links and published API/SDK documentation; do not repurpose OHub as the website |
| OBackup and its recovery boundary | Backup protocols and verification tooling, without moving backup secrets or recovery authority into Engine |

OID should remain a separately deployable identity service behind OEnd's one public API gateway. OEnd and OID should share contracts and routes, not be collapsed into one Worker or repository. OPoint, Omy Play and O remain separate products with their own authorization and data authority.

The canonical Omy Play source repository in the connected organization is `OtyaHQ/Omy`; `Omy_Play` is a historical repository slug/reference. New links and integrations should use the canonical `Omy` repository while preserving existing product/API compatibility paths.

## Application access model

```text
Browser / mobile / external client
              |
              v
       Product frontend
     (OWeb / OPoint / Omy / O)
              |
              v
       OEnd versioned API
              |
       private service calls
              |
       +------+----------------+
       |                       |
       v                       v
 Product-owned service   Otya Engine host
       |                       |
       v                       v
 Product-owned data       reusable engine
 and authorization        capabilities
```

A product can also embed the Rust library directly where appropriate. Non-Rust applications should use a versioned HTTP/service-binding client once that public host contract is implemented. Websites remain frontend applications; Otya provides their backend/runtime capabilities, not a forced website UI framework.

## First integration: OEnd preview canary

The first OEnd integration is tracked in [OEnd PR #418](https://github.com/OtyaHQ/OEnd/pull/418).

It introduces a separate `oend-engine-preview` Worker with a private Service Binding to `otya-engine-edge-preview`. Its only route, `GET /_internal/engine/status`, is protected by a preview-only bearer secret and checks the Engine's health, version and non-sensitive `/capabilities` document, including the application API version.

The canary:
- proves private Worker-to-Worker connectivity;
- tests the Engine runtime, application API version, declared capability contract and fail-closed behavior;
- does not accept customer or product requests;
- does not read/write product data;
- does not modify production Workers, service bindings, routes, DNS, domains, databases, queues or secrets.

The Otya Engine preview Worker is defined in this repository's manual deployment workflow. Both the Otya and OEnd workflows must be reviewed/merged before they become available on the default branch. The preview deployment is a separate manual action and must not be confused with production deployment.

## Adoption sequence

| Order | Product | First safe use | Explicitly retain |
|---|---|---|---|
| 1 | OEnd | Private runtime canary; then non-financial shared operation/event capabilities after the async contract exists | Gateway, OID authorization, product service ownership, existing D1 and queues |
| 2 | OPoint | Staging integration for generic operations, idempotency utilities and auditable background work after conformance tests | Company membership, branch permissions, stock, cash, double-entry ledger, provider credentials and payment state |
| 3 | Omy Play | Staging integration for connected catalogue/update/asset metadata operations where it improves reliability | Local-first playback, local library, media lifecycle, OID identity and Omy-owned data |
| 4 | O | Owner-only status and approved operational tasks through existing OEnd authorization | Owner memory, privileged actions, approval policy and device secrets |
| 5 | OWeb | Build the public site separately; connect its server-side routes to OEnd and approved Engine capabilities | Public UI, SEO, content publishing, browser security and accessibility |
| 6 | OHub | Document supported Engine APIs/modules and developer starter projects | Public documentation ownership and publication safety |

OHub does not need to run an Engine instance merely because it documents Otya. Flutter clients should continue calling the product's supported API; they do not link the Engine's Wasm Worker.

## Runtime selection

- **Cloudflare Workers/WebAssembly:** short request/response work and host-provided bindings. The current adapter exposes health/version/capability metadata plus a read-only sample; it is not yet a production business API. The preview Worker may expose this non-sensitive demo on `workers.dev`; the production Engine Worker has `workers_dev = false` and is reached only through OEnd's private Service Binding.
- **Linux native library/service:** first full runtime for filesystem, SQLite and longer-running operations.
- **OCI container/VPS:** later Linux deployment profile where a full process environment is needed.
- **Other hosts:** add adapters only after host contract conformance tests exist.

Phone access is the operator workflow: GitHub review, CI and manual deployment can be used from a browser. The engine itself remains headless and runs on a host independently of the operator's phone.

## Non-negotiable safety and ownership rules

1. One Otya Engine installation belongs to one operator boundary. Do not introduce a central Otya customer directory, mandatory Engine account, vendor telemetry or hidden control plane.
2. Each product owns its user/business authorization and authoritative data. An Engine module must not bypass OID, OPoint company/branch scope or O's owner approval gate.
3. Financial records stay in OPoint's authoritative store. Do not move the ledger to a shared Engine database, cache or queue.
4. Omy playback and local media remain usable without cloud connectivity.
5. Browser clients call same-origin frontend routes or the documented public API. They never receive service-binding access, database credentials or provider secrets.
6. Keep existing production Workers and domains intact until a separately reviewed migration proves parity, rollback and data recovery.
7. Preview and production resources, credentials, queues and databases must remain isolated.

## Current Engine limitations before product traffic

The current SDK has a synchronous exact-route registry and static application manifests. Its Cloudflare adapter currently passes anonymous requests to a read-only catalogue example; it does not authenticate product users or forward request bodies. The portable Storage contract is synchronous and the SQLite adapter is not yet a complete business schema/backup solution.

Before OEnd handles real application traffic, implement and test:
- an asynchronous host/storage contract suitable for D1 and other async adapters;
- authentication/session verification integration that constructs trusted caller context;
- request-body, header and response adaptation with bounded streaming;
- idempotent durable operations, outbox/inbox, retries, leases, scheduler and reconciliation;
- module lifecycle, schema migrations, backup/restore and adapter conformance;
- observability, limits, abuse controls, recovery and cross-runtime integration tests.

The first canary is intentionally limited to health/version. It is evidence of a safe integration boundary, not proof that Otya Engine already powers production business operations.

## Verified repository and platform inventory

At review time, the connected GitHub account exposes repositories for Otya Engine, OPoint, OEnd, O, Omy Play, OHub and OWeb. OWeb has no committed files yet; OHub contains documentation and should not be repurposed as the website.

The connected Cloudflare account contains the existing product Workers `oend`, `oid`, `opoint`, `omy`, `o` and `obackup`. No Cloudflare Pages projects were returned in the account inventory. The active zone is `otya.site`. This inventory is read-only evidence, not permission to rename, delete or reroute existing resources.

## Pilot links

- [Otya Engine draft PR #2](https://github.com/OtyaHQ/Otya/pull/2)
- [OEnd preview integration draft PR #418](https://github.com/OtyaHQ/OEnd/pull/418)
