# Otya Platform Roadmap

This roadmap is milestone-based, not date-based. A milestone is complete only when its acceptance evidence exists.

## Phase 0 — Foundation and decisions

**Goal:** agree on the boundaries before implementation expands.

Deliverables:
- Platform architecture.
- Otya Language 0.1 draft.
- Linux-first kernel decision.
- Rust bootstrap and self-hosting plan.
- Initial security model.
- Repository licensing and contribution decision.

Exit gate:
- Architecture reviewed.
- Language proposal reviewed.
- Licensing and contribution policy approved before external code is accepted.

## Phase 1 — Minimal bootstrap toolchain

**Goal:** execute a tiny, well-defined Otya program on a development host.

Deliverables:
- Rust workspace and CLI.
- .ot lexer and parser.
- Type checking for the supported language subset.
- Source diagnostics.
- First compiler target.
- Unit, parser and negative tests.

Exit gate:
- Documented example compiles and runs in a clean CI environment.
- Invalid programs fail with useful diagnostics.
- Supported target and host versions are explicit.

## Phase 2 — Runtime and standard library

**Goal:** run portable Otya modules with enforced permissions.

Deliverables:
- Runtime host using the selected Wasm engine.
- Versioned host interfaces.
- Capability grants and resource limits.
- Initial standard-library modules.
- Test and debugging support.

Exit gate:
- Denied capabilities are actually denied.
- Resource limits are verified.
- Runtime compatibility and failure behavior are tested.
- No unsupported security guarantee is claimed.

## Phase 3 — Engine services and package lifecycle

**Goal:** make Otya Engine useful on an existing supported Linux distribution.

Deliverables:
- Engine service manager and health reporting.
- Package manifest and dependency resolution.
- Package integrity and signature model.
- Install, update, rollback and removal flows.
- otya doctor and recovery diagnostics.

Exit gate:
- Fresh install, upgrade, rollback and uninstall pass in isolated test hosts.
- User data is preserved as documented.
- Interrupted installs recover safely.
- Packages cannot exceed their granted permissions.

## Phase 4 — Otya OS composition

**Goal:** assemble a bootable Otya operating environment.

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

## Phase 5 — Self-hosting and ecosystem

**Goal:** reduce bootstrap dependence and open a safe contribution model.

Deliverables:
- Otya compiler implementation in .ot.
- Repeatable self-hosting pipeline.
- Stable public SDK and extension interfaces.
- Contributor governance and review rotation.
- Public package and documentation infrastructure.

Exit gate:
- Reproducible bootstrap evidence.
- Security response and release process operational.
- Compatibility policy published.
- External contributions pass the same review and CI requirements as maintainer code.

## Always-on quality requirements

Every phase must preserve:

- Backward compatibility or a documented migration path.
- Automated tests and clean-build evidence.
- Security review for permissions and native code.
- Dependency and artifact provenance.
- Performance measurement on declared reference hardware.
- Installation and recovery documentation.
- Clear distinction between implemented, tested and deployed features.

## Explicit non-goals for the first release

- Writing a replacement kernel before the engine has proven its value.
- Replacing the Otya ecosystem's existing backend, identity or product databases.
- Giving ordinary apps unrestricted root or host access.
- Claiming every third-party API or service will work automatically.
- Publishing a production or security certification without evidence.
