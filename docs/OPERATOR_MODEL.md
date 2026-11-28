# Customer-Operated Engine Model

## Core rule

The person or organization installing Otya Engine owns and operates the installation. Otya supplies software and documentation; Otya does not host, monitor, back up, administer or control the customer's environment by default.

## Customer-owned responsibilities

The operator selects and manages:
- host hardware, operating system, network, DNS and firewall;
- installation mode, configuration, service availability and capacity;
- database or storage provider, schema backup and restore destination;
- provider accounts, billing, API credentials and secret rotation;
- user accounts, organization membership, role assignment and access review;
- domain, TLS certificates, email identity, sender verification and consent;
- retention, export, deletion and legal compliance for their workloads;
- patching and updates when using a self-managed release channel;
- incident response, disaster recovery, logs and monitoring;
- authorization of all business operations and provider activity.

## Otya software responsibilities

The publisher remains responsible for its own software distribution and stated support commitments, including:
- accurate documentation of supported versions and known limitations;
- secure handling of build/release signing keys;
- publishing fixes and security advisories according to its policy;
- not silently collecting or transmitting customer data;
- documenting upgrade, migration, backup and recovery behavior;
- respecting third-party licenses and intellectual property;
- maintaining a private vulnerability reporting route.

A contract cannot erase non-waivable legal duties. Responsibility follows actual control, processing and applicable law; a disclaimer is not a substitute for sound architecture.

## No hidden control plane

By default:
- no Otya-hosted database or storage;
- no Otya account is required;
- no license check or remote kill switch is required for local operation;
- no background telemetry or update check without opt-in;
- no customer secrets copied to Otya;
- no remote support access;
- no automatic deployment or provider-side mutation.

If optional telemetry or update discovery is introduced, it must be opt-in, explain the exact fields and destination, and continue to allow local operation when disabled.

## Data ownership and portability

Customer records remain in the configured customer-owned store. Provide documented export formats and import tools. Backups include a manifest with engine version, schema version, module versions, selected configuration metadata and checksums; they must not include raw provider secrets by default.

Do not encrypt backups with an Otya-only key. The customer controls the recovery key. Losing the customer's keys may make encrypted backups unrecoverable; document that clearly.

## Shared installation and security

Embedded mode inherits the host application's process boundary. Local service mode runs under a dedicated unprivileged OS account and uses authenticated local IPC. Multi-user or multi-tenant access must be explicitly configured; do not assume that local OS access implies business authorization.

A customer may request optional support. Support access must be temporary, visible, least-privilege, revocable and logged by the customer's own environment.

## Responsibility matrix

| Activity | Customer/operator | Otya publisher |
|---|---|---|
| Hardware, OS, network, DNS | Owns and operates | Documents requirements |
| Local configuration and credentials | Owns and operates | Provides safe interfaces |
| Provider account and provider contract | Owns and operates | Maintains optional adapter |
| Customer backup schedule and restore destination | Owns and verifies | Provides tools and tested procedures |
| Engine package build and published signatures | Verifies before install | Produces and protects release artifacts |
| Engine defects and security advisories | Applies available updates | Triage, disclose and publish fixes per policy |
| Business authorization and lawful use | Owns decisions and operations | Provides technical controls and clear limitations |
| Optional support access | Approves and revokes | Uses only approved, time-limited access |

This is a product responsibility model, not a legal opinion or a blanket transfer of all liability.
