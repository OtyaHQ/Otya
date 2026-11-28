# One Installation, One Operator

## Product invariant

An Otya Engine installation is a local component controlled by one operator. The operator may be an individual or an organization. It is not a tenant inside a central Otya SaaS service.

**There is no global or cross-installation company, customer, tenant, account or end-user directory.** One installation may own its private local identity realm and project registry.

The engine does not register the operator with a vendor service or share people, projects or resources with another installation. The installation may record its own organization/workspaces/projects and operate its own staff identity directory, login UI and access policies. It has no vendor account, central registration or remote heartbeat.

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

## Installation-owned identity; application-owned authorization

The full Engine distribution may provide a private identity provider, sign-in portal, user/group directory, MFA, sessions and OIDC/OAuth clients for the installing operator. These identities, signing keys and sessions exist only inside that installation. No Otya-hosted or cross-company user directory is involved.

Applications may reuse that identity provider for staff SSO, but each application decides its own roles, data access, financial permissions and business rules. Authentication by itself never grants a business action. A public product may maintain a separate consumer identity system when its audience and trust boundary differ.

The local service socket is protected by OS permissions. Processes under the same OS user may have access to that user's socket and files. It is not a sandbox between processes sharing one UID.

## Consequences

- No multi-tenant SaaS architecture in the engine core.
- No global or cross-installation customer/identity database.
- No vendor analytics identifying installations.
- No Otya-hosted login required; each installation may operate its own private Otya identity issuer.
- No automatic remote enable/disable.
- No shared service for unrelated operators.
- Simpler local-first operation and clearer ownership.
- Host applications retain their own resource authorization even when using installation-owned sign-in.
