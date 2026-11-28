# Engineering Influences and Otya Design Rules

## Purpose

Otya should study mature systems deeply and adopt the engineering principles that made them dependable, portable and widely usable. We are not copying Linux, Python, Rust, systemd, SQLite or OCI, and we are not rebuilding those projects inside Otya.

**Reuse proven operating-system capabilities and open standards. Build Otya's own engine contracts, modules, installation experience and integration logic.**

This keeps Otya independent while avoiding the cost and risk of reinventing infrastructure that already works.

## Reference systems and what Otya should learn

| System | What it does well | What Otya should adopt | What Otya should not become |
|---|---|---|---|
| Linux kernel and userspace | Kernel/userspace boundary, process and filesystem primitives, documented interfaces, broad architecture support | Run as a normal userspace engine; use OS permissions, processes, sockets, signals and resource controls; keep platform code behind narrow interfaces | A new kernel, Linux distribution, kernel module collection or replacement scheduler |
| Unix and POSIX | Small composable tools, files, streams, processes, signals and stable conventions | CLI tools that compose; explicit stdin/stdout/stderr behavior; graceful signal handling; portable filesystem/process abstractions | A clone of the shell or a new POSIX implementation |
| Python | Readable public APIs, broad standard library, discoverable modules, packaging, virtual environments and embedding interfaces | A clear SDK, useful first-party modules, clean package boundaries, optional Python bindings and predictable dependency isolation | A mandatory Python runtime, Python interpreter fork or a second business-logic implementation |
| Rust and Cargo | Memory safety, explicit ownership, crate reuse, feature selection, workspaces, target-aware builds and API compatibility rules | Rust as the canonical core; small crates; stable interfaces; minimal unsafe code; optional features; explicit MSRV and target matrix | A new compiler or language runtime in this repository |
| systemd | Process supervision, restart policy, logs, watchdogs and resource controls | Let the host service manager supervise the optional daemon; integrate with journal and signals; expose health/readiness; test resource limits | A process supervisor or init system built into Otya |
| SQLite and PostgreSQL | Transactions, durability, concurrency controls and operationally understood database behavior | Define clear storage contracts and use database-specific adapters; test crash/recovery and backup/restore | An Otya database engine or an assumption that SQLite is distributed |
| OCI | Interoperable image, runtime and distribution standards | Offer standard container images later as one deployment option; preserve a native install path | A container runtime, private image format or required container dependency |
| WebAssembly component ecosystem | Explicit component interfaces and potential isolation across language/toolchain boundaries | Evaluate a sandboxed extension ABI after core contracts and security model are mature | A mandatory Otya language or a claim that WebAssembly alone makes code safe |

## 1. Linux: use the operating system, do not replace it

Otya is a userspace engine. Linux already handles process scheduling, memory management, file permissions, users and groups, sockets, signals, filesystems and many isolation/resource primitives.

Otya should:
- run without root for normal embedded and per-user operation;
- use the installing operator's OS security boundary;
- separate configuration, durable data, state, cache and runtime sockets;
- use Unix-domain sockets for optional local IPC;
- use systemd for optional service supervision, restart policy and journal integration;
- detect whether cgroups, namespaces, seccomp, AppArmor or SELinux are available before relying on them;
- avoid kernel changes, privileged daemons and public listeners by default.

Linux is the first implementation target, not the only possible target. The Rust core should avoid Linux-only assumptions. Filesystem, service manager, process, IPC and secret-store behavior belong behind platform-specific modules.

## 2. Python: learn packaging and breadth, not runtime dependence

Python's strength is not only its syntax. Its standard library, module import model, virtual environments, packaging ecosystem and extension/embedding interfaces make it easy to integrate.

Otya should learn from those ideas:
- one discoverable package per capability;
- stable public APIs and clear version requirements;
- optional modules rather than one giant dependency bundle;
- isolated dependency environments for Python consumers;
- a thin Python SDK over the Otya Rust API when justified;
- examples and developer documentation that make integrations easy.

The Rust engine must not require Python to be installed. Do not embed CPython in the core. Do not write the same business feature once in Rust and again in Python. Python is an optional wrapper/client language for application developers, not a competing core runtime.

## 3. Rust: one canonical engine, modular crates

Rust is the canonical implementation language for the core. Use a Cargo workspace with clear boundaries:
- engine-core: lifecycle, typed errors, capability contracts and stable interfaces;
- engine-config: schema parsing, defaults and migrations when the boundary is useful;
- engine-storage: storage traits and transaction contracts;
- engine-runtime: jobs and optional service runtime;
- engine-cli: operator-facing commands;
- platform modules: Linux first, then other supported operating systems;
- adapter crates: provider-specific integrations;
- SDK crates: host-language bindings.

Do not split crates merely to create a large directory tree. A crate is justified when it has a cohesive API, independent testing/release value or a meaningful optional-dependency boundary.

Use Cargo features for optional integrations, not hidden runtime downloads. Avoid unsafe Rust by default; every necessary unsafe block must be narrow, documented and covered by tests. Public APIs need an explicit compatibility and deprecation policy.

## 4. Batteries included, but not forced

A powerful engine should be useful immediately, but installing the core must not download every provider SDK or force every module on every operator.

The distribution model should have three layers:

1. **Small core:** configuration, lifecycle, errors, capability enforcement, diagnostics and extension contracts.
2. **First-party optional modules:** storage adapters, scheduler, communications, audit, jobs and domain modules installed only when required.
3. **External adapters:** operator-selected providers and services with explicit credentials and scopes.

The operator should be able to inspect enabled modules, permissions, versions, dependencies and storage before starting them. Removing a module must never silently delete its data.

## 5. Portability: design for it, prove it one target at a time

Keep business/domain logic independent of operating-system APIs. Put differences in platform modules. Avoid hard-coded Linux paths in engine-core. Use explicit build targets and CI for each claimed platform.

Initial supported targets:
- Linux x86_64;
- Linux aarch64.

After the Linux install, upgrade, service, backup and recovery lifecycle is proven, add Windows and macOS. Consider BSD or other targets only when there is an owner and repeatable CI/testing evidence. A successful cross-compile alone is not proof that installation, service management, files or IPC work.

For containers, use OCI-compatible images later where they help operators. Native installation and embedding must remain possible without Docker or Kubernetes.

## 6. Reliability is an engine feature

Adopt the proven operational lessons of databases and service managers:
- transactions instead of hand-written multi-step persistence;
- durable outbox/inbox patterns for external effects;
- idempotency and explicit unknown outcomes;
- bounded retry policies and provider-aware rate limits;
- startup recovery and graceful shutdown;
- health/readiness checks;
- checksummed backup and restore tests;
- migrations with preflight, locking and interruption recovery;
- structured logs that do not expose secrets.

Never promise exactly-once delivery to an external provider unless the provider contract actually guarantees it. Prefer honest status and reconciliation over false success.

## 7. Interoperability: standard interfaces before custom protocols

Prefer existing standards where they meet the requirement:
- C ABI for stable native language bindings;
- HTTP/REST and signed webhooks for service integrations;
- SMTP for portable email transport;
- S3-compatible APIs for object storage;
- SQLite/PostgreSQL drivers for persistence;
- OCI for future container distribution;
- systemd and the platform's native service manager for process lifecycle.

Create Otya-specific formats only when the existing standard does not express the engine's actual contract. Version and document any custom format from its first release.

## 8. Distribution and trust

A user should be able to install Otya without creating an account with Otya.

The release should provide:
- architecture-specific binaries and packages;
- checksums and cryptographic signatures;
- a software bill of materials;
- release notes and support matrix;
- offline verification and install paths;
- migration and rollback guidance;
- no mandatory phone-home, account activation or telemetry.

The update command may contact the operator-selected release source only when the operator requests or schedules an update. It must verify the artifact before applying it.

## 9. Performance and footprint

Keep the base engine small and measure it. Do not optimize by removing safety, durability or diagnostics. Maintain benchmarks for startup time, idle memory, binary size, throughput and latency. Publish size numbers by target and feature set, because optional adapters change the final artifact.

The early small-binary goal is a measured budget for the minimal core, not a promise that every adapter and business module fits in the same binary size.

## 10. Source references

These primary references inform the design. They are references, not code to copy:

- Linux kernel documentation: https://docs.kernel.org/
- Python standard library: https://docs.python.org/3/library/
- Python virtual environments: https://packaging.python.org/en/latest/specifications/virtual-environments/
- Python extending and embedding: https://docs.python.org/3/extending/
- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Cargo semantic-versioning guidance: https://doc.rust-lang.org/cargo/reference/semver.html
- Cargo platform targets: https://doc.rust-lang.org/cargo/reference/targets.html
- systemd service unit semantics: https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html
- SQLite transactions: https://www.sqlite.org/lang_transaction.html
- SQLite write-ahead logging: https://sqlite.org/wal.html
- OCI image specification: https://specs.opencontainers.org/image-spec/

## Decision

Otya will be **Rust-first, Linux-first, local-first, modular, standards-based and independently installable**. Python and other languages are supported through optional SDKs after the core ABI is stable. The engine remains useful offline for local workloads, without Otya registration or a shared company/tenant control plane.
