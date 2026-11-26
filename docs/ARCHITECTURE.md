# Otya Platform Architecture

**Status:** proposed architecture for the experimental foundation.

## 1. Product model

Otya is one platform with multiple cooperating layers:

1. Kernel and hardware foundation.
2. Otya Engine.
3. Otya Runtime and language toolchain.
4. System services and standard interfaces.
5. Applications, packages and developer tools.
6. Otya OS distribution and user experience.

Otya Engine is independently installable. Otya OS is the complete operating environment that includes the engine and additional system components.

## 2. Kernel and hardware foundation

### Initial choice: Linux

The initial supported target is Linux. The kernel provides low-level mechanisms that an operating environment needs, including process and memory management, scheduling, device drivers, filesystems and networking.

Otya will not fork or replace the Linux kernel for the first milestone. The project should first prove that its engine and language offer a coherent, useful platform above a supported kernel.

### Hardware interface

Otya should expose stable, documented system interfaces instead of requiring every Otya application to call Linux-specific functions directly.

The engine maps those interfaces to the host kernel and supported devices. Hardware-specific code stays in reviewed system components and drivers; ordinary applications receive only the capabilities granted to them.

## 3. Otya Engine

The engine is the installable platform foundation. Its responsibilities are:

- Starting, supervising and stopping Otya services.
- Loading and running Otya packages.
- Enforcing declared application capabilities and resource budgets.
- Providing stable filesystem, network, clock, process and device interfaces.
- Managing package installation, verification, update and rollback.
- Recording useful, privacy-conscious diagnostics.
- Exposing versioned local system APIs.
- Providing health checks and recovery paths.

The engine must remain useful on an existing supported Linux distribution. Installing it must not require replacing the user's operating system.

## 4. Otya Language and compiler

Otya source uses the .ot extension. The language is intended for application code, system automation and eventually selected platform services.

The compiler is responsible for:

- Lexing and parsing source files.
- Producing clear source locations and diagnostics.
- Type checking and validating module imports.
- Enforcing language safety rules.
- Compiling supported programs to the initial portable target.
- Recording compiler and target versions in build metadata.

The language specification is versioned separately from implementation details. Syntax changes must have compatibility notes and tests.

## 5. Runtime and execution model

### Initial portable target

The initial application target is WebAssembly. The runtime should align with the WebAssembly Component Model and WASI, so separately compiled modules can use explicitly defined interfaces.

Wasmtime is the first runtime candidate because it supports embedding WebAssembly execution, WASI and the Component Model in host applications. It remains an implementation choice that must be validated for Otya's performance, platform and security requirements.

### Capability-based access

Applications receive no ambient access to the whole machine.

A package declares the capabilities it needs. The installer or system administrator reviews and grants those capabilities. The runtime exposes only the approved interfaces.

Examples of capability categories include:

- Read or write access to specific application data directories.
- Network access to declared endpoints or approved network scopes.
- Notifications.
- Audio and media devices.
- Selected system information.
- User-approved device operations.

The exact capability vocabulary, consent flow and enforcement rules require a separate security specification. A manifest declaration is a request, not automatic authorization.

### Resource limits

The runtime must support configurable limits for execution time, memory, CPU use, concurrency and I/O where the host runtime permits them. Limits and failures must be observable and testable.

A sandbox reduces risk; it does not make every program safe. Native extensions, host interfaces, dependencies and the kernel remain part of the threat model.

## 6. System APIs and services

Otya system APIs must be:

- Versioned.
- Documented with inputs, outputs, permissions and failure modes.
- Backward-compatible within a declared compatibility window.
- Covered by conformance tests.
- Explicit about whether an operation is local, remote or provider-dependent.

The engine must distinguish a local platform guarantee from an external service result. It must not claim that a third-party API, webhook, payment provider or identity service is functioning merely because a client can be configured.

Provider adapters should report unknown outcomes honestly. Retries must be safe and idempotent where the provider contract allows it.

## 7. Packages and installation

### Package metadata

The proposed package manifest is otya.toml. It identifies the package, version, entry point, runtime target, dependencies and requested capabilities.

The proposed archive extension is .otpkg. Its binary layout, signature rules, compression, content addressing and repository protocol are not stable yet.

### Engine installation

The engine installer should support a minimal install on a supported Linux distribution, validate host compatibility, preserve existing user data, verify downloaded artifacts and provide a documented removal and recovery path.

### Otya OS installation

A later Otya OS image can bundle the supported Linux kernel, boot and system services, Otya Engine, standard library, user experience and selected applications.

An installable engine kit and a bootable Otya OS image are related deliverables, not the same artifact.

## 8. Developer experience

The planned CLI is otya. Its intended commands include:

- otya new — create a project.
- otya check — parse, type-check and validate a project.
- otya build — compile it for a supported target.
- otya run — run a project under the development runtime.
- otya test — run project tests.
- otya fmt — format source.
- otya package — build a distributable package.
- otya install — install a package or, in a separate documented mode, the engine.
- otya doctor — diagnose host and engine configuration.

These are planned command names, not currently available commands.

## 9. Security boundaries

The initial threat model must include malicious or compromised packages, dependency substitution, compromised update repositories, hostile input, resource exhaustion, unsafe native extensions and vulnerabilities in the runtime or host kernel.

Minimum design requirements:

- Deny-by-default application capabilities.
- No root privileges for ordinary applications.
- Verified package and update metadata.
- Reproducible or independently verifiable build artifacts where practical.
- Resource limits and bounded logs.
- Explicit administrator approval for privileged system changes.
- Security review for native extensions and kernel-facing components.
- Clear reporting and response procedures for vulnerabilities.

Do not claim the platform is secure until the relevant controls have been implemented and independently tested.

## 10. Existing Otya ecosystem boundary

The engine is not a replacement for the existing Otya backend or product databases.

OEnd, OID and the individual Otya products retain their current ownership and authorization rules. Any future bridge must be a separately documented, versioned integration with least-privilege permissions.

Otya Engine should first work as a standalone platform. Ecosystem integration is an optional layer built on stable public interfaces, not a dependency required to boot or use the engine.

## 11. Quality gates

A component is not considered ready because its source code exists. Each release must include evidence for:

- Unit and integration tests.
- Compiler and runtime conformance tests.
- Supported-host and hardware matrix.
- Security and dependency checks.
- Installation, update, rollback and uninstall tests.
- Compatibility tests between engine, runtime, language and package versions.
- Performance and resource measurements.
- Recovery documentation and a tested recovery procedure.

The release status must distinguish design-complete, implemented, CI-tested, manually verified and production-deployed.
