# Contributing to Otya

Otya is at the experimental architecture stage. Design feedback is welcome.

## Before submitting code

The repository's license and contributor agreement have not yet been approved. Public repository visibility is not an open-source license. Until the licensing and governance policy is published, do not assume that a pull request can be merged or that submitted code can be redistributed.

For now, use issues and proposed design documents to discuss:
- Language syntax and semantics.
- Runtime and system API design.
- Security boundaries.
- Linux integration and supported hardware.
- Build, test and recovery requirements.

Do not submit code copied from projects unless its license and attribution requirements have been checked.

## Engineering expectations

When code contributions are opened for review:

- Keep changes focused and explain the problem being solved.
- Include tests for behavior and error cases.
- Update the relevant architecture or language specification.
- Document platform assumptions and new permissions.
- Avoid adding dependencies without explaining their purpose and maintenance status.
- Never include credentials, private keys, tokens, customer data or production secrets.
- Do not bypass CI, security checks or review requirements.
- Do not make production deployment or compatibility claims without evidence.

## Proposed review model

The intended model is maintainer-reviewed pull requests, automated checks, documented design proposals for major changes and explicit security review for privileged components.

The final contributor agreement, code ownership, release authority, voting and dispute processes will be published before external code contributions are enabled.
