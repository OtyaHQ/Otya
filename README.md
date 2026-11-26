# Otya

**Otya is an integrated systems platform, language and operating environment.**

> Status: experimental foundation. This repository currently establishes architecture and language direction. It is not a released operating system, production engine, or security-certified runtime.

## One platform, several layers

Otya is developed as one coordinated platform with clearly separated components:

- **Otya Engine** — the installable systems foundation: runtime hosting, services, package lifecycle, permissions, system APIs and device integration.
- **Otya Language (.ot)** — the native developer-facing language for applications, services, automation and eventually parts of the platform itself.
- **Otya Runtime** — loads and executes compiled Otya modules under explicit resource limits and capability permissions.
- **Otya Toolchain** — compiler, formatter, test runner, package manager, diagnostics and developer CLI.
- **Otya OS** — a complete operating environment that combines the engine, system services, user experience and a supported kernel.

These are components of one platform, not separate products that need unrelated repositories.

## The Linux relationship

The initial Otya OS target will use the Linux kernel as its hardware and kernel foundation. Otya will build its own platform layer above it.

That gives the project access to an established kernel and driver ecosystem while it develops its own language, engine, system interfaces, packages, desktop/user experience and installation experience.

Otya Engine should also be installable on a supported existing Linux distribution. A later Otya OS image can bundle the engine and the rest of the Otya environment into a bootable installer.

A custom kernel is a possible future research direction, not a prerequisite for the first useful release. Replacing Linux should require a clear technical reason, a maintained driver strategy and measurable benefits.

## Initial technical direction

- Bootstrap implementation: Rust.
- Otya source files: .ot.
- Initial portable application target: WebAssembly, with interfaces shaped for the WebAssembly Component Model and WASI.
- Initial embeddable runtime candidate: Wasmtime.
- Initial OS foundation: Linux.
- Package manifest: otya.toml.
- Planned distributable package extension: .otpkg (not yet a stable format).

Wasmtime is designed to run WebAssembly, WASI and the Component Model outside browsers. Its documentation describes portable system interfaces and component composition. See the [Wasmtime documentation](https://docs.wasmtime.dev/) and [Component Model documentation](https://component-model.bytecodealliance.org/).

Rust is the bootstrap implementation language, not the language developers must use to build Otya applications. See [The Rust Programming Language](https://doc.rust-lang.org/book/).

These are starting decisions for the prototype. They must be validated with working code, benchmarks, compatibility tests and security review before being treated as permanent standards.

## Example direction

The following is proposed Otya 0.1 syntax. It is a language design example, not a claim that the compiler already runs it.

~~~ot
module hello;

use std::io;

fn main() {
    io::println("Hello from Otya!");
}
~~~

The .ot extension identifies Otya source code. Configuration and package metadata remain structured data files such as otya.toml; not every file needs to use .ot.

## Target repository layout

~~~text
otya/
  compiler/       # lexer, parser, type checker and code generation
  runtime/        # Wasm execution and capability host
  engine/         # services, system APIs and lifecycle management
  std/            # Otya standard library
  tools/          # CLI, formatter, test runner and package manager
  sdk/            # public interfaces and development tooling
  os/             # Linux integration and Otya OS composition
  installer/      # engine installer and future OS image tooling
  docs/           # architecture, language and security specifications
  examples/       # small .ot programs
  tests/          # compiler, runtime and compatibility tests
~~~

This is the intended architecture, not yet the implemented directory tree.

## Boundaries with the existing Otya ecosystem

Otya Engine is a platform foundation. It does not replace or absorb the existing Otya products and their data ownership.

- OEnd remains the existing shared backend and service integration layer.
- OID remains the existing identity and account-security service.
- Omy Play remains the local-first media product.
- Opoint remains the private company operations product.
- O remains the private owner intelligence application.
- OHub remains the public developer and partner documentation surface.

The engine may provide versioned local and remote integration interfaces, but it must not silently become a second identity provider, business database, or shared backend. Existing product boundaries and permissions remain authoritative.

Third-party APIs and services are integrations, not guarantees. An adapter must declare its permissions, version, authentication requirements, health checks and failure behavior. The engine must not claim that an external API is correct, legal, available or compatible merely because an adapter exists.

## Development status

The first milestones are architecture and language specification, a Rust bootstrap compiler, a constrained runtime, a small standard library and reproducible tests. The installable engine comes after those core contracts work.

No release should be described as production-ready until its supported platforms, installation and recovery flows, upgrade behavior, security boundaries, compatibility policy and test evidence are published.

## Licensing and community status

This repository is currently public, but it does not yet contain an approved open-source license or contribution agreement. Public visibility alone does not grant permission to reuse, redistribute or modify the code.

The license, trademark policy, contributor process and governance model must be approved and published before accepting external code contributions. Architecture proposals and issue discussions can help shape the project in the meantime.

## Start here

- [Platform architecture](docs/ARCHITECTURE.md)
- [Otya Language 0.1 draft](docs/LANGUAGE_V0_1.md)
- [Bootstrap and self-hosting](docs/BOOTSTRAP_AND_SELF_HOSTING.md)
- [Roadmap and acceptance gates](docs/ROADMAP.md)
- [Platform foundation decision](docs/decisions/0001-platform-foundation.md)
- [Contributing guidance](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
