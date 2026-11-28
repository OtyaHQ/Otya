# Otya Engine Security Policy

**Status:** preliminary policy for an experimental, customer-operated Rust engine. This policy is not a security certification or a guarantee that all deployments are secure.

## Reporting a vulnerability

Please do not publish credentials, personal data or working exploit details in a public issue.

Use GitHub's private vulnerability reporting feature for this repository when enabled. If it is not available, contact the maintainers through an existing trusted private channel and request a secure reporting route. Do not send secrets through public discussions.

A useful report identifies:
- affected component and exact engine version or commit;
- prerequisites and steps to reproduce in a safe test environment;
- expected and observed behavior;
- security impact and affected trust boundary;
- whether the issue affects embedded mode, local service mode, an adapter or the installer;
- a safe reproduction or validation method.

## Security boundaries

The engine is designed to run in the customer's environment. Its threat model includes:
- malicious or compromised host applications and modules;
- malicious configuration and untrusted webhook/provider input;
- local users attempting to cross application or tenant boundaries;
- stolen or over-scoped provider credentials;
- compromised dependencies, build workflows or release artifacts;
- unsafe filesystem paths, local IPC and service permissions;
- resource exhaustion, disk-full and denial-of-service conditions;
- update, migration, backup and restore failures;
- vulnerabilities in the operating system, database and third-party providers.

## Required engineering controls

The project must implement and test:
- default-deny module capabilities and least privilege;
- unprivileged service execution and restrictive data/config/socket permissions;
- secret references and redaction from logs, errors and diagnostic bundles;
- authenticated local IPC and safe network defaults;
- webhook signature, freshness, replay and request-size validation;
- dependency and license review, SBOM and build provenance;
- signed release artifacts with verification before installation and updates;
- bounded resource use, queue retries and provider calls;
- backup integrity checks and tested restore procedures;
- private vulnerability handling and security patch communication.

## Privacy and telemetry

The default engine must work without contacting Otya infrastructure. Telemetry, remote diagnostics, update discovery and support access are disabled unless the customer explicitly enables them. Diagnostic exports must be reviewed by the operator and redact credentials, tokens, OTPs and unnecessary personal data.

## Security claims

A passing build or test suite does not prove the engine is secure. Every security claim must identify the exact release, feature boundary, supported platform, test method and remaining limitations.

Do not commit secrets, API tokens, private keys, customer data, recovery keys or production credentials to source files, examples, test fixtures or build artifacts.
