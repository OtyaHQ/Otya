# Decision 0001: Independently Installable Engine Foundation

- Status: accepted for the engine-only project direction
- Scope: Otya Engine repository
- Revisit when: measured compatibility, security or operational needs require a change

## Context

The initial repository proposal combined an installable engine, a programming language, a compiler, a portable runtime and a complete operating environment. Building all of these together created unnecessary dependencies and made the product boundary unclear.

The current product objective is a reusable engine kit that a customer installs into and operates within their own environment. The design should follow the useful packaging lesson of Omy Play's embedded Media Kit dependency: a shared API and core implementation with optional platform-specific components, without requiring a vendor-hosted service for local functionality.

## Decision

1. This repository develops Otya Engine only.
2. The engine is implemented in Rust.
3. The primary integration is an embeddable Rust library.
4. An optional local service and operator CLI provide shared background operations and lifecycle management.
5. The customer supplies and owns the host environment, data, storage, credentials, provider accounts, backups and operation.
6. No Otya-hosted compute, mandatory central database or Otya cloud account is required.
7. Storage, identity, messaging, provider, filesystem and network integrations are explicit, replaceable adapters.
8. Cloudflare, GitHub, Resend and Firebase are optional provider integrations, not engine runtime dependencies.
9. The project does not develop an Otya language, compiler or language runtime.
10. Linux is the first supported service-installation target. The engine must also be embeddable by supported host applications.
11. OEnd and the existing Otya ecosystem remain separate. Engine integration into those products is optional and must respect their current boundaries.

## Consequences

Benefits:
- A narrower product that can be built, tested and installed incrementally.
- No need to operate a vendor cloud to make local functionality work.
- One reusable Rust core across embedded and service modes.
- Provider changes do not require rewriting business modules.
- Customer data and infrastructure stay under customer control.

Constraints:
- Customers must provide and operate their infrastructure and configured storage.
- Self-hosting transfers operational work to the customer; it does not make operations automatic.
- The engine must provide excellent diagnostics, upgrade, backup and recovery tools.
- Native Rust extensions require trusted builds; untrusted extensions need an isolated process or sandbox boundary.
- No production or security claim is valid without implementation, test and review evidence.

## Revisit criteria

Reconsider the decision only with evidence showing:
- a customer requirement that cannot be met by the current embedded/service model;
- a documented threat model and operational cost;
- compatibility and migration impact;
- a testable implementation and rollback strategy.
