# Decision 0001: Linux-First Platform Foundation

- Status: accepted for the experimental prototype
- Scope: initial Otya Engine and Otya OS architecture
- Revisit when: measured technical requirements show that the current foundation blocks a necessary capability

## Context

Otya aims to combine an installable systems engine, its own programming language, a portable runtime, developer tooling and a complete operating environment.

Building a new general-purpose kernel, driver ecosystem, language, compiler, runtime, package system and user experience at the same time would create a large number of unproven dependencies before users can test a useful result.

The project needs a usable foundation that can be developed and tested incrementally.

## Decision

1. Use the Linux kernel as the initial Otya OS kernel foundation.
2. Build Otya Engine as a distinct, installable platform layer above Linux.
3. Make the engine usable on supported existing Linux distributions before producing a complete Otya OS image.
4. Develop .ot as Otya's own source language.
5. Bootstrap the first compiler and engine components in Rust.
6. Target WebAssembly initially, with runtime interfaces designed around WASI and the Component Model.
7. Evaluate a custom kernel only after the Otya platform has measurable requirements that Linux cannot satisfy adequately.

## Consequences

Benefits:
- Earlier access to real hardware and a mature driver ecosystem.
- Engine installation can be tested without replacing a user's operating system.
- Language, runtime, package and system API work can proceed in small increments.
- The OS image becomes a composition of tested components rather than one inseparable prototype.

Costs and constraints:
- The first Otya OS is Linux-based and must comply with the licenses of its included components.
- Some low-level behavior remains defined by Linux and the host distribution.
- Otya-specific hardware and security interfaces must be designed above the kernel.
- The Wasm runtime and component interfaces add dependencies that require version and security management.

## Revisit criteria

Reconsider the kernel foundation only with a written proposal that provides:
- A capability Linux cannot provide adequately for Otya's defined goals.
- A realistic driver, hardware and compatibility strategy.
- A threat model and maintenance plan.
- Measured evidence that the proposed alternative improves the required outcome.
- A migration plan that does not strand existing applications or users.
