# One Installation, One Operator

## Product invariant

An Otya Engine installation is a local component controlled by one installing operator. It is not a tenant inside a central Otya SaaS service.

**There is no core concept of company, customer, tenant, account or end-user.**

The engine does not ask who the installing person works for, which company uses it, how many people use the host application or where that person registers. It has no vendor account, central registration, remote heartbeat or installation inventory.

## What is identified locally

The engine reads only what it needs to run:
- execution mode (embedded or per-user service);
- configured storage adapter and path;
- enabled local modules and their permissions;
- provider adapters explicitly configured by the operator.

The filesystem root, process owner and OS user provide the installation boundary. A local path or OS UID is not exported as a global customer identity.

## Isolation model

Separate installations must not share:
- configuration files;
- SQLite databases or PostgreSQL schemas by default;
- secret-store namespaces;
- Unix socket paths;
- log/state directories;
- module enablement or migration state;
- backup manifests or encryption keys.

The operator can run one installation at a time or deliberately manage several separated installations. The CLI must make the target config/data root visible before destructive actions.

## Identity belongs above the engine

If an application needs accounts, roles, company membership, customers or tenants, that application implements and operates them. It may pass a scoped, short-lived authorization decision to a module operation, but the engine core does not create a user directory or decide the application's business identity.

The local service socket is protected by OS permissions. All processes under the same user may have access to that user's socket and files. It is not a sandbox between processes sharing one UID.

## Consequences

- No multi-tenant SaaS architecture.
- No central customer database.
- No vendor analytics identifying installations.
- No Otya login required.
- No automatic remote enable/disable.
- No shared service for unrelated OS users.
- Simpler local-first operation and clearer ownership.
- Host applications must implement their own user authorization.
