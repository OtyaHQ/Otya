# Media Kit Design Lessons for Otya Engine

## Reference inspected

Omy Play's current architecture document identifies `media_kit` as its shared audio/video playback engine. Its `pubspec.yaml` separates the primary `media_kit` package, video presentation package and native media libraries.

The upstream Media Kit project describes a modular package structure, shared cross-platform implementation, native libmpv integration through FFI and platform-specific video embedding. See:
- https://github.com/media-kit/media-kit
- https://github.com/media-kit/media-kit/blob/main/README.md

This is a packaging and integration reference. Otya must not copy its code or assume its license permits reuse; the project should use its own implementation and separately reviewed dependencies.

## What Otya should learn

1. **One predictable API.** An application uses one stable engine API rather than learning provider-specific internals.
2. **Split packages by capability.** Keep core small and add optional modules only when the application needs them.
3. **Shared implementation.** Put common logic in the Rust core and keep language/platform wrappers thin.
4. **Native code behind a boundary.** Use explicit FFI ownership and lifecycle rules; avoid duplicating the engine in each client language.
5. **Platform-specific pieces stay separate.** A video-rendering bridge or mobile-specific integration should not force every consumer to install every component.
6. **Local work remains local.** Media playback works without a Media Kit hosted control plane; Otya local engine functions should have the same independence.
7. **Test the release build.** Native libraries, ABI, architecture and packaging must be validated in release-mode applications, not only in unit tests.

## Proposed Otya package experience

For a Rust consumer:

```toml
[dependencies]
otya-engine-core = "..."

# Add only if needed:
otya-engine-sqlite = { version = "...", optional = true }
otya-engine-resend = { version = "...", optional = true }
```

For a Flutter consumer, the eventual package set might look like:

- `otya_engine`: Dart API and lifecycle wrapper;
- `otya_engine_native`: Rust native library bridge;
- optional provider/storage packages selected by the host application.

These are naming concepts only; no crates or packages with these names are claimed to exist yet.

## What not to copy

- Do not bundle all providers and databases into a mandatory giant install.
- Do not make every feature a separate daemon or network service.
- Do not expose raw native handles without ownership/lifetime rules.
- Do not put provider secrets inside mobile app packages.
- Do not confuse a portable library with a hosted backend.
- Do not mistake easy installation for operational reliability: upgrades, data migrations, backups, restore and support lifecycle still need separate design.

## Otya's distinct value

Media Kit solves media playback. Otya Engine aims to solve reusable local systems infrastructure: lifecycle, permissions, storage adapters, durable operations, provider connectors, business modules and diagnostics. It should borrow the modular packaging discipline, not the problem domain.
