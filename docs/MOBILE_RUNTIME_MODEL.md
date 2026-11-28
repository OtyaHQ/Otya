# Otya Mobile Runtime Model

**Status:** architecture defined; Android core compile checks are being added; no Android/iOS application package or FFI SDK is released yet.

## Two different meanings of mobile support

Otya has to support both:

1. **A person who only has a phone and wants to develop or administer a project.** Use GitHub's browser editor for small edits, Codespaces for remote Rust compilation and terminal work, GitHub Actions for automated builds/tests, and the operator's cloud dashboard/CI for deployments. See [Phone-First Development](PHONE_FIRST_DEVELOPMENT.md).
2. **An Otya-powered application that people use on a phone.** The application embeds the portable Otya Rust core through a reviewed FFI boundary or uses a remote Otya service operated by its owner. The user should not be asked to install a desktop CLI, Rust compiler or Linux distribution on a phone.

These are separate products and release gates. A successful cloud build workflow does not itself create a mobile application.

## Android-first application architecture

The preferred first mobile target is Android arm64, with x86_64 available for emulator/CI work. The portable Rust core remains free of Android UI and Android-specific framework dependencies.

The intended stack is:

- **Host application:** Kotlin/Java Android app or a Flutter/Dart app.
- **Engine API:** versioned, safe C ABI from the Rust core, with generated/maintained platform bindings. Flutter uses a thin Dart FFI package; a native Android app uses a narrow JNI/Kotlin bridge where required.
- **Platform adapter:** maps app lifecycle, filesystem locations, secure storage and background scheduling to Android's supported APIs.
- **Optional storage:** a future Android-compatible SQLite adapter, separately packaged from the Linux adapter.
- **Remote integrations:** optional HTTP/webhook/provider adapters configured by the app operator, never hard-coded into the core.

Do not expose Rust pointers, internal structs, allocator ownership or panic behavior directly across FFI. The ABI must use opaque handles, versioned structs/functions, explicit buffer ownership, bounded inputs, deterministic error codes, panic containment and lifecycle tests. No raw engine secret or access token should be shipped in the app binary.

## Android platform rules

Android is not a small Linux server. Do not reuse the Linux CLI/service model unchanged.

- Store local engine data inside the application's private storage sandbox using paths supplied by Android's application context. Do not assume XDG directories or a shared filesystem path.
- Use the Android Keystore-backed platform adapter for keys and sensitive local credentials where appropriate. Do not put secrets in ordinary TOML, preferences or logs.
- Use Android's lifecycle and background-work facilities (for example, WorkManager for deferrable reliable work). Do not promise an always-running daemon; Android can suspend or terminate apps.
- Network activity must be explicit, permissioned, observable and resilient to connectivity changes.
- Queue work locally only when the feature has a clear retention, retry, idempotency and user-consent policy.
- The app owns its user identities and authorization. The engine must not create a shared customer, tenant or end-user identity service.
- Offline operation should be supported for features whose data and business rules are local; features requiring a remote provider must display the unavailable/pending state rather than claim success.

## Linux and Android must remain distinct

The current Linux installation layer uses XDG paths, directory file descriptors and Linux-specific ownership/mode checks. The current SQLite adapter also uses Linux-specific filesystem preparation. Those modules are not automatically available on Android simply because Android uses the Linux kernel.

Android needs its own platform path, secret, lifecycle and storage adapters. Reuse the portable core contracts; do not bypass the Android application sandbox or assume root access.

## Cloudflare and web applications

Cloudflare Workers run in Cloudflare's V8/WebAssembly environment, not as arbitrary native Linux processes. The Workers runtime does not accept native-code binaries; Rust can be compiled to WebAssembly for supported Worker use cases. Workers also do not provide a normal local filesystem.

Therefore:
- use Workers for optional edge APIs, web applications, webhooks and coordination;
- use D1, KV, R2, Queues, Workflows and Durable Objects only when their consistency and lifecycle match the module's contract;
- use a separate Linux service/host when a workload needs native OS facilities, long-running processes or a normal filesystem;
- consider a pure portable WebAssembly core later, with a dedicated Worker platform adapter;
- never describe deploying a Worker as installing the native Otya Linux engine.

A phone user can access a responsive Otya-powered web app through the browser, or a native Android app when one is released. Neither requires the user to own a PC.

## iOS after Android

iOS follows the same portable-core/host-adapter separation, with a reviewed Swift/Objective-C FFI layer, iOS sandbox paths, Keychain-backed secrets, and background task rules. iOS builds, signing, simulator tests and App Store delivery require Apple-specific CI and release setup. It is a later platform gate, not claimed by Android target compilation.

## Mobile release gates

1. Public versioned Rust C ABI with opaque handles, error codes, buffer lifecycle and thread-safety rules.
2. Android path/secret/lifecycle platform adapter and private app storage handling.
3. Android SQLite adapter with migration, file permissions, integrity and offline tests.
4. Kotlin/JNI and Dart FFI packages with API compatibility tests.
5. Android arm64 builds using the Android NDK and reproducible package assembly.
6. Instrumented tests on emulator/device for lifecycle, process death, background work, network loss, permissions and upgrade.
7. Signed build artifacts, SBOM, provenance, release notes and supported Android-version matrix.
8. iOS adapter and platform-specific CI only after Android's boundaries are proven.

The current Android workflow only checks that the platform-neutral core compiles for Android Rust targets. It does not pass these release gates and does not mean the engine is ready for phone production use.
