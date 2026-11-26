# Otya Security Policy

**Status:** preliminary policy for an experimental project. Otya is not yet a production operating system or security-certified runtime.

## Reporting a vulnerability

Please do not publish credentials, personal data, or working exploit details in a public issue.

Use GitHub's private vulnerability reporting feature for this repository when it is enabled. If private reporting is not available, contact the repository maintainers through an existing trusted private channel and ask for a secure reporting route. Do not send secrets through public discussions.

A report should include:
- Affected component and version or commit.
- Conditions required to reproduce the issue.
- Expected and observed behavior.
- Security impact, without exposing third-party data.
- A safe way for maintainers to reproduce or validate the report.

## Initial security priorities

The project must design and test controls for:

- Package provenance and update integrity.
- Runtime sandboxing and application capabilities.
- Resource exhaustion and denial of service.
- Dependency and build-system compromise.
- Native extensions and host interface boundaries.
- Privileged engine operations.
- Installer, update and rollback integrity.
- Data isolation and safe diagnostics.

## Security claims

A successful build or test run does not establish that the engine is secure. Security claims must identify the exact version, tested boundary, test method and remaining limitations.

Do not store secrets in source files, package manifests, examples, build artifacts or issue attachments.
