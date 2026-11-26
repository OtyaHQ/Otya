# Bootstrap and Self-Hosting Strategy

**Status:** proposed engineering plan.

## Why a bootstrap is necessary

A compiler written in Otya cannot compile itself until an Otya compiler already exists. The project therefore needs a small, trusted first implementation written in an established language.

This is normal compiler engineering, not a compromise in the long-term design.

## Stage 0 — Rust bootstrap

Build the first toolchain in Rust.

The initial bootstrap should contain:

- The otya command-line program.
- A lexer and parser for the supported .ot subset.
- A type checker and useful diagnostics.
- A code generator for the chosen portable target.
- A small development runner and test harness.

The bootstrap should be intentionally small. Keep unsafe Rust out of the core unless a reviewed, documented reason requires it.

Rust's official language book and reference are the baseline implementation references: [Rust Book](https://doc.rust-lang.org/book/) and [Rust Reference](https://doc.rust-lang.org/reference/).

## Stage 1 — First usable Otya compiler

The Rust compiler accepts the published Otya language subset and compiles .ot projects to the initial WebAssembly target.

The release gate requires:

- Language conformance tests.
- Stable diagnostics.
- Standard-library integration.
- Reproducible test fixtures.
- A working build and run loop.
- Clear host/runtime compatibility checks.

At this stage, Otya programs are written in .ot, while the compiler itself is still written in Rust.

## Stage 2 — Compiler implementation in Otya

Once the language is sufficiently expressive, implement a second compiler in .ot.

This compiler must be able to compile the same published language specification. It must not depend on undocumented behavior or privileged interfaces available only to itself.

Keep the Rust compiler available as the recovery and reference implementation.

## Stage 3 — Bootstrap comparison

Use a staged build:

1. Rust compiler builds the Otya compiler written in .ot.
2. The resulting Otya compiler builds its own source again.
3. The outputs are compared using a documented reproducibility rule.
4. The complete compiler and runtime test suites run against the rebuilt compiler.
5. A clean environment repeats the process from pinned source and dependencies.

A successful command is not sufficient proof of self-hosting. The resulting compiler must pass the same conformance, security and compatibility gates as the bootstrap compiler.

## Stage 4 — Self-hosted release

Otya may be called self-hosted only after:

- The Otya compiler can build itself from source.
- The bootstrap process is documented and repeatable.
- Independent clean builds produce equivalent, explainable artifacts.
- The self-hosted compiler passes all required tests.
- The Rust bootstrap remains available for recovery.
- A release can be reproduced without relying on an undocumented developer machine.

The first self-hosted release may still use Rust for selected runtime, installer or platform components. Self-hosting the language compiler does not require rewriting the entire kernel or every system component in Otya.

## Principles

- Never delete the known-good bootstrap before its replacement has passed release gates.
- Pin toolchain and dependency versions for release builds.
- Preserve compiler source, release artifacts and build provenance.
- Test compiler output, not just compiler process exit codes.
- Treat compiler changes as security-sensitive platform changes.
