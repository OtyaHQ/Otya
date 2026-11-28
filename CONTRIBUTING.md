# Contributing to Otya Engine

Otya Engine is an experimental Rust-based, independently installable and embeddable engine kit.

## Scope

This repository is for:
- Rust core, local service and operator CLI;
- installation, packaging, configuration and lifecycle;
- storage, transaction, module and provider-adapter contracts;
- security, reliability, backup, recovery and observability;
- native SDK and FFI integrations.

This repository does not develop a programming language, compiler, language runtime or operating-system distribution.

## Before submitting code

The repository's license and contributor agreement must be approved before external code contributions can be accepted. Public repository visibility does not by itself grant reuse or redistribution rights.

Until the licensing and governance policy is published, use issues and proposed design documents to discuss:
- engine architecture and APIs;
- installation and supported platforms;
- storage and recovery guarantees;
- provider adapters and module boundaries;
- security and operational controls.

Do not submit code copied from projects unless its license and attribution requirements have been checked.

## Engineering expectations

- Keep changes focused and explain the operational problem being solved.
- Include unit, integration and negative tests.
- Test failures, restarts, interrupted operations and recovery where relevant.
- Update the engine architecture, API contract and operator documentation.
- Document platform assumptions, data handling and permissions.
- Avoid adding dependencies without explaining their purpose, license and maintenance status.
- Never include credentials, private keys, tokens, customer data or production secrets.
- Do not bypass CI, security checks or review requirements.
- Do not claim production readiness, security or provider compatibility without evidence.
- Preserve customer-owned data and provide a migration/rollback plan for schema changes.

## Proposed review model

Use maintainer-reviewed pull requests, automated checks, written design proposals for architectural changes and explicit security review for privileged components. Publish code ownership, release authority, security response and contributor governance before enabling external contributions.
