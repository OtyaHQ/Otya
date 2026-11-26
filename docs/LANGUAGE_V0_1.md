# Otya Language 0.1 — Draft

**Status:** design draft. Syntax and semantics are not frozen. The compiler does not yet implement this specification.

## 1. Source files

- Otya source files use the .ot extension.
- Source text is UTF-8.
- A package may contain multiple .ot modules.
- Package metadata uses otya.toml, not .ot.
- The language version and compiler version must be recorded in build metadata.

The .ot extension is the standard source extension for Otya code.

## 2. Design goals

The first language version should be:

- Readable and predictable.
- Statically typed.
- Suitable for application and system-service development.
- Portable across supported Otya targets.
- Designed around explicit error handling.
- Safe by default, with privileged operations available only through explicit interfaces.
- Small enough to implement, test and document rigorously.

The first version should prioritize clear semantics over a large number of language features.

## 3. Proposed example

~~~ot
module hello;

use std::io;

fn main() {
    io::println("Hello from Otya!");
}
~~~

This example illustrates the intended style. It is not executable until the parser, compiler, standard library and runtime implement the required features.

## 4. Initial language surface

### Modules and imports

A module begins with an optional module declaration. Imported modules use explicit paths.

~~~ot
module sample;

use std::io;
~~~

Imports must resolve to modules available to the project or its declared dependencies. The compiler must reject unresolved and unauthorized imports.

### Functions

Functions use the fn keyword and explicit parameter and return types.

~~~ot
fn add(left: I32, right: I32) -> I32 {
    return left + right;
}
~~~

The initial release should support a single well-defined entry point for executable packages.

### Variables

Bindings are immutable by default. Mutation requires an explicit mutable declaration.

~~~ot
let label: Text = "Otya";
let mut counter: I32 = 0;
counter = counter + 1;
~~~

The compiler rejects assignment to an immutable binding.

### Basic types

The initial type inventory should be deliberately small:

- Bool
- I32, I64
- U32, U64
- F32, F64
- Text
- Bytes
- Unit
- List<T>
- Map<K, V>
- Option<T>
- Result<T, E>

The final numeric overflow, conversion, string encoding and collection semantics must be specified before the language is frozen.

### Control flow

The initial subset should include if / else, while, bounded iteration, match and return. Loop behavior and pattern matching rules must be covered by executable tests.

### Errors

Expected failures should use explicit result values rather than silently disappearing.

~~~ot
fn read_label() -> Result<Text, ReadError> {
    // Standard-library implementation will define the actual operation.
}
~~~

The exact syntax for propagation and error construction is reserved for a later language proposal.

## 5. Standard library

The standard library should provide stable, portable interfaces for:

- Text and collections.
- Serialization.
- Time and clocks.
- Logging and diagnostics.
- Filesystem access through granted capabilities.
- Networking through granted capabilities.
- Process and task coordination.
- Testing.

The language specification defines semantics. The standard library defines APIs. The runtime provides the host implementation. These layers must not be conflated.

## 6. Permissions and capabilities

Importing a module does not silently grant access to the machine.

For example, a filesystem module may expose operations, but the runtime must still check whether the application has the required filesystem capability and whether the requested path is inside its grant.

The package manifest declares requested permissions. The system or user authorizes them according to policy. The runtime enforces the grant.

The first release should not include unrestricted raw system calls or general-purpose unsafe memory operations in ordinary application code.

## 7. Compilation and execution

The initial implementation is expected to use a Rust bootstrap compiler and compile supported .ot programs to WebAssembly.

The runtime will host the compiled program and expose approved interfaces through the Component Model/WASI design.

The language must not expose runtime-specific implementation details as permanent language guarantees. This keeps the language portable if the underlying runtime changes.

## 8. Grammar outline

The following is an intentionally incomplete grammar outline. It is a guide for implementation, not a final parser specification.

~~~text
program       = module_decl? import_decl* item*
module_decl   = "module" path ";"
import_decl   = "use" path ";"
item          = function_decl
function_decl = "fn" identifier "(" parameters? ")" return_type? block
parameters    = parameter ("," parameter)*
parameter     = identifier ":" type
return_type   = "->" type
block         = "{" statement* "}"
statement     = let_statement
              | assignment
              | expression_statement
              | return_statement
              | if_statement
              | while_statement
              | block
let_statement = ("let" | "let" "mut") identifier type_annotation? "=" expression ";"
~~~

The parser specification must define comments, literals, operator precedence, escapes, whitespace, source locations and error recovery before 0.1 is frozen.

## 9. Versioning and compatibility

- Language syntax and semantic changes require a versioned proposal.
- Breaking changes require migration notes and updated examples.
- The compiler must reject unsupported language versions with a clear diagnostic.
- Standard-library APIs and system interfaces are versioned independently.
- A compiler release must state the language versions it supports.

## 10. Conformance suite

Before the language can be called 0.1 stable, it needs:

- Lexer tests for every token and invalid character sequence.
- Parser tests for valid and invalid programs.
- Type-checker tests, including negative tests.
- Runtime tests for defined behavior.
- Golden diagnostic tests for source spans and error messages.
- Standard-library contract tests.
- Cross-platform output consistency tests.

No feature is part of the language merely because one compiler build happens to accept it. The written specification and conformance suite are authoritative.
