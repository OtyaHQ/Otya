# One Installation, One Operator

## Product invariant

An Otya Engine installation is a local component controlled by one operator. The operator may be an individual or an organization. It is not a tenant inside a central Otya SaaS service.

**There is no core concept of company, customer, tenant, account or end-user.**

The engine does not ask who the installing operator works for, which company uses it, how many people use the host application or where that operator registers. It has no vendor account, central registration, remote heartbeat or installation inventory.

## What is identified locally

The engine reads only what it needs to run:
- execution mode (embedded, per-user service or operator-managed server service);
- configured storage adapter and path;
- enabled local modules and their permissions;
- provider adapters explicitly configured by the operator.

The operating-system security principal and local installation paths provide the installation boundary. A local path, process UID or optional local nickname is not exported as a global customer identity. No remote installation ID is required.

## Isolation model

Separate installations must not share by default:
- configuration files;
- SQLite databases or PostgreSQL schemas;
- secret-store namespaces;
- Unix socket paths;
- log/state directories;
- module enablement or migration state;
- backup manifests or encryption keys.

The operator can run one installation or deliberately manage several isolated installations. The CLI must make the target config/data root visible before destructive actions. If the operator intentionally connects two installations, that connection is an explicit operator-managed integration, not automatic shared state.

## Identity belongs above the engine

If an application needs accounts, roles, company membership, customers or tenants, that application implements and operates them. It may pass a scoped, short-lived authorization decision to a module operation, but the engine core does not create a user directory or decide the application's business identity.

The local service socket is protected by OS permissions. Processes under the same OS user may have access to that user's socket and files. It is not a sandbox between processes sharing one UID.

## Consequences

- No multi-tenant SaaS architecture in the engine core.
- No central customer database.
- No vendor analytics identifying installations.
- No Otya login required.
- No automatic remote enable/disable.
- No shared service for unrelated operators.
- Simpler local-first operation and clearer ownership.
- Host applications must implement their own user authorization.
