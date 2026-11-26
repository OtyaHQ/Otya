# Bootstrap Compiler 0.1 — Implementation Boundary

**Status:** implementation candidate on the bootstrap compiler branch. CI results determine whether this implementation is accepted.

## What it implements

The Rust bootstrap provides a small end-to-end compiler and bytecode virtual machine. It is intentionally dependency-free.

The supported source subset includes:

- Optional module and use declarations.
- One parameterless main entry point.
- Immutable and mutable local bindings.
- I32, I64, Int, Text, String and Bool type names (integer aliases share a signed 64-bit runtime representation).
- Integer arithmetic and comparisons.
- Boolean equality and inequality.
- Unary integer negation and boolean not.
- io::println for integer, text and boolean values.
- return from main.
- UTF-8 string literals with basic escapes.

The compiler performs static type checks for the supported subset. The VM checks bytecode stack and local-slot safety, and arithmetic uses checked operations.

## Commands implemented

- otya new <name>
- otya check [file.ot|project-directory]
- otya build [file.ot|project-directory]
- otya run [file.ot|file.otbc|project-directory]
- otya doctor
- otya --version

Build writes a versioned textual bytecode artifact under .otya/build/. Run can execute source directly or execute a previously built .otbc artifact.

## What is not implemented

This milestone is not yet:

- A WebAssembly compiler or WASI runtime.
- A full Otya Language 0.1 implementation.
- A multi-module linker or general function-call implementation.
- A package manager or signed package format.
- A system service manager or Linux daemon.
- An installer or bootable OS image.
- A security boundary for hostile bytecode or applications.

The initial bytecode format is a development artifact. Do not distribute it as a stable package format or treat it as a security boundary.

## Next acceptance gates

1. CI formatting, compiler tests and Clippy pass.
2. Source and bytecode smoke tests produce the exact expected output.
3. Parser and compiler diagnostics gain source spans throughout the AST.
4. The bytecode verifier validates every instruction and local slot before execution.
5. The compiler and VM are separated into library modules with a stable test API.
6. The next backend proposal compares WebAssembly Component Model/WASI with a versioned Otya VM target using measured portability, performance, diagnostics and sandboxing results.

A feature is implemented only when its behavior is covered by tests and its supported limits are documented.
