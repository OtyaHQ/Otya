# Otya Platform Roadmap

This roadmap is milestone-based, not date-based. A milestone is complete only when its acceptance evidence exists.

## Phase 0 — Foundation and decisions

Deliverables: platform architecture, Linux-first decision, Otya Language 0.1 draft, bootstrap/self-hosting plan, security model, licensing and contribution decision.

Status: architecture documents are committed. Licensing and contribution governance remain open.

## Phase 1 — Bootstrap compiler vertical slice

Goal: compile and execute a small .ot program on a development host.

Deliverables:
- Rust CLI and dependency-free compiler.
- Lexer, parser and static checks for a deliberately small subset.
- Versioned development bytecode and VM.
- Source, build-artifact and error-path tests.
- CI formatting, tests, lint and smoke checks.

Status: implementation candidate is on feat/bootstrap-compiler-v0. Its initial CI run passed formatting, all compiler tests, strict Clippy, source checking, source execution, bytecode building and compiled-bytecode execution. Review and additional verifier/security work remain before this milestone is accepted as a stable compiler.

Exit gate:
- Clean CI build with no warnings.
- All compiler and VM tests pass.
- Source and compiled-bytecode examples produce identical expected output.
- Unsupported syntax fails with a clear diagnostic.

## Phase 2 — Compiler correctness and runtime boundary

Goal: strengthen the bootstrap before adding a portable production target.

Deliverables:
- Source spans for every AST node and diagnostic.
- Bytecode verifier with stack and slot analysis.
- Separate compiler and VM libraries.
- Fuzz/property tests for lexer, parser, serializer and verifier.
- A documented runtime threat model.
- A measured choice between a portable Otya VM target and WebAssembly/WASI.

Exit gate:
- Malformed bytecode is rejected before execution.
- Error locations are accurate.
- CI covers positive, negative and adversarial inputs.

## Phase 3 — Language 0.1 and portable runtime

Goal: implement the reviewed language subset and a versioned, portable execution contract.

Deliverables:
- Frozen grammar and semantic rules.
- Standard library modules and API versioning.
- Portable target backend.
- Capability-based host interfaces.
- Resource budgets and runtime conformance suite.

Exit gate:
- Language conformance suite passes.
- Denied capabilities are actually denied.
- Host/runtime compatibility and failure behavior are tested.

## Phase 4 — Otya Engine on Linux

Goal: make the engine installable on existing supported Linux distributions.

Deliverables:
- Engine lifecycle and supervised services.
- Package metadata, dependency resolution and artifact verification.
- Install, update, rollback and removal flows.
- Health reporting and recovery diagnostics.
- Supported-host matrix and documented system APIs.

Exit gate:
- Fresh install, upgrade, rollback and uninstall pass in isolated test hosts.
- User data is preserved as documented.
- Interrupted updates recover safely.
- No ordinary app receives root privileges.

## Phase 5 — Otya OS composition

Goal: assemble a bootable Otya operating environment.

Deliverables:
- Supported Linux kernel and boot configuration.
- Otya Engine and required system services.
- Base user experience and system settings.
- Installer/image builder.
- Hardware compatibility matrix.
- Update, recovery and accessibility experience.

Exit gate:
- Clean installation and upgrade on declared hardware.
- Boot, storage, networking, suspend/resume and peripheral tests.
- Recovery from a failed update.
- Published release notes and support policy.

## Phase 6 — Self-hosting and community

Goal: reduce bootstrap dependence and enable safe contributions.

Deliverables:
- Otya compiler implementation in .ot.
- Repeatable self-hosting pipeline.
- Stable public SDK and extension interfaces.
- Approved license, trademark policy and contributor terms.
- Maintainer review, security response and release governance.
- Public package and documentation infrastructure.

Exit gate:
- Reproducible bootstrap evidence.
- Security response and release process operational.
- Compatibility policy published.
- External contributions pass the same review and CI requirements as maintainer code.

## Always-on quality requirements

Every phase preserves backward compatibility or a documented migration path, automated tests and clean-build evidence, security review for privileged code, dependency provenance, performance measurements, installation and recovery documentation, and honest separation between implemented, tested and deployed features.

## Explicit non-goals for the first release

- Writing a replacement kernel before the engine has proven its value.
- Replacing Otya's existing backend, identity or product databases.
- Giving ordinary apps unrestricted root or host access.
- Claiming every third-party API or service will work automatically.
- Publishing production or security certification without evidence.
