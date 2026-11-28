# Single-Operator Installation Model

## Core rule

One Otya Engine installation belongs to one operator. The operator may be a person or an organization. The complete distribution is a local, self-hosted application platform, not a hosted SaaS account and not a shared multi-tenant service.

The installation may maintain its own local operator/workspace/project records, identity realm, staff/user directory, login portal, groups, service clients and application registrations. Those records exist only inside that operator's installation. Otya does not register customers, share identities across installations, count remote users or report installations to a central Otya service.

The operator owns the local installation. Its built-in identity provider may authenticate people across selected company apps, while each host application remains responsible for its product-specific roles, resource permissions, business identity and authorization.

## What one installation means

- One installation has one operator-controlled configuration and data root.
- It runs inside the selected OS security principal's boundary.
- Its local database, credentials, logs, cache, runtime socket and backups belong to that installation.
- A host application may embed the library, or the operator may enable one local service.
- Other OS users do not automatically share the installation. They install and operate their own separate instance if they need it.
- A server operator may explicitly select a dedicated unprivileged service account for one isolated installation.
- The operator may deliberately create multiple isolated installations, but each has separate configuration, state, process/socket and storage. The engine never merges them into a tenant pool.

A company or organization may choose to install and operate Otya for its own use. That does not turn the engine into a multi-tenant SaaS product. Its application data model is chosen by the host application or installed business module, not inferred or registered by the engine.

## No global or cross-operator control plane

The product must not contain:
- a global Otya customer or user directory shared across installations;
- cross-operator tenant provisioning or tenant routing;
- mandatory Otya-hosted identity login or remote installation registration;
- vendor-controlled account suspension or a remote kill switch;
- a mandatory hosted control plane or hosted database.

The local installation may contain its own identity service, user/group directory, operator console, workspace/project registry and local management API. Local accounts belong to that installation only. The Engine identity service authenticates principals; project applications still authorize business actions and resource access.

## Customer/operator responsibilities

The operator selects and manages:
- installation identity policy, owner recovery methods, staff invitations, role assignments and external identity federation;
- the host computer/server, OS account, OS updates and network;
- installation mode and service lifecycle;
- local configuration, database and filesystem permissions;
- provider accounts, billing, API credentials and secret rotation;
- business application user accounts, roles and access review;
- domains, TLS, email sender setup, consent and provider contracts;
- data retention, export, deletion and applicable compliance;
- backup schedules, separate backup destinations and restore drills;
- monitoring, capacity, incident response and disaster recovery;
- authorization of business actions and provider activity.

## Otya software publisher responsibilities

The publisher remains responsible for its own software distribution and stated support commitments, including:
- accurate documentation of supported versions and limitations;
- secure handling of release signing keys;
- publishing fixes and security advisories according to its policy;
- no silent collection or transmission of customer data;
- documented upgrade, migration, backup and recovery behavior;
- third-party license and attribution compliance;
- a private vulnerability reporting route.

A contract cannot erase non-waivable legal duties. Responsibility follows actual control, processing and applicable law; a disclaimer is not a substitute for sound architecture.

## No hidden control plane

By default:
- no Otya-hosted database, storage, compute or control plane;
- no Otya account or login;
- no registration, heartbeat or remote instance inventory;
- no telemetry or update check without opt-in;
- no customer secrets copied to Otya;
- no remote support access;
- no automatic provider or infrastructure mutations;
- no network listener in embedded mode.

Optional update discovery or telemetry must describe exact data, purpose and destination, remain disabled by default, and never be required for local operation.

## Local data ownership and portability

The operator controls local records and configured stores. Provide documented export/import paths. Backups include the engine version, module versions, schema versions, timestamps and checksums, but exclude raw provider secrets by default.

Do not encrypt backups with an Otya-only key. The operator controls recovery keys. Losing the operator's keys may make encrypted backups unrecoverable; explain this plainly.

## Linux process boundary

The default Linux service target runs in the installing user's own service manager and uses that user's XDG directories. A later server deployment may use one dedicated unprivileged service account selected by the operator. Neither mode is a shared tenant-hosting service.

## Responsibility matrix

| Activity | Operator | Otya publisher |
|---|---|---|
| Hardware, OS, network and capacity | Owns and operates | Documents supported requirements |
| Local configuration, files and credentials | Owns and operates | Provides safe interfaces |
| Provider accounts and contracts | Owns and operates | Maintains optional adapter |
| Backups and restore destination | Owns and verifies | Provides tools and tested procedures |
| Release artifacts and signing | Verifies before installation | Produces and protects artifacts |
| Engine defects and advisories | Applies updates as appropriate | Triage, disclose and publish fixes per policy |
| Application resource permissions and business authorization | Host application/operator owns | Engine supplies identity and policy primitives, not product decisions |
| Optional support access | Approves and revokes | Uses only approved, time-limited access |

This is a product responsibility model, not a legal opinion or blanket transfer of all liability.
