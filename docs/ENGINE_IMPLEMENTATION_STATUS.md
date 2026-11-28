# Otya Engine Implementation Status

**Status date:** 2026-11-28

## Implemented in the engine-only branch

- Repository scope set to Otya Engine only; old language/compiler scope removed from the active branch.
- Rust workspace with core, CLI and separate optional Linux SQLite adapter crates.
- Strict TOML configuration parser and semantic validation.
- Configuration has no company, tenant, customer or end-user identity fields; unknown identity fields are rejected.
- Local CLI commands: `otya version`, `otya doctor`, `otya paths`, `otya config init`, `otya config validate`, `otya storage init --config FILE`, and `otya storage check --config FILE`.
- Linux XDG path discovery for per-operator config, data, state, cache and optional runtime directories.
- Named local instances with validated names and separate config/data/state/cache/runtime path namespaces.
- Explicit `otya paths --create` action to prepare the selected installation directories.
- Linux filesystem preparation walks paths through directory file descriptors, refuses symlink components, rejects unsafe parent directories and foreign-owned final directories, refuses root execution, and applies owner-only 0700 permissions to final Otya directories.
- Filesystem tests cover directory create/idempotency, permission hardening, symlink rejection, unsafe writable parent rejection, relative paths, private-file mode 0600, existing-file refusal, symlink-target refusal and invalid file names.
- CLI smoke test creates an isolated named instance on a non-root Linux runner.
- Read-only `otya paths` inspection does not create or modify files.
- Linux `otya config init` prepares only the config directory and exclusively creates `otya-engine.toml` with mode 0600; it refuses existing files and does not follow symlinked parents.
- The starter config selects embedded mode with memory storage and emits a volatility warning.
- The core exports a backend-neutral storage/transaction contract with bounded namespace, key, value and prefix-scan limits.
- The separate Linux SQLite adapter implements that contract using private operator-owned files, no-follow checks, Otya database identity metadata, schema v1-to-v2 migration, WAL, bounded busy timeout and rollback-safe transactions.
- CLI `otya storage init --config FILE` explicitly initializes the SQLite database; `otya storage check --config FILE` checks an existing database without creating a missing one or running a migration.
- A phone-accessible Codespaces devcontainer provides a remote Rust environment with SQLite build dependencies and Android/WebAssembly Rust targets.
- A separate portability workflow compiles the platform-neutral core for Android and WebAssembly; these are library compatibility checks only, not an APK or app build.
- A minimal Cloudflare Rust Worker adapter and manual deployment workflow are committed; the new dedicated CI check and a first preview deployment are pending.
- Phone-first and headless-runtime architecture guides distinguish remote development/control from the engine's runtime.
- A separate Cloudflare Workers Rust/WebAssembly host adapter serves /health and /version, a responsive static example website and the example catalog API; Worker compile checks passed on run 90. A first deployed preview and browser end-to-end test have not been performed.
- The application SDK, example application and route-level tests are committed: manifests, exact routes, application API version checks, host-supplied caller context, access policies, declared capability checks and request/response bounds.
- Initial provider metadata contracts added to the Rust core: provider kind/auth method, credential-store references (no raw credential value field), provider connection, project/environment resource binding and project capability grant. Validation rejects missing credential references, duplicate scopes/references and grants that do not match the selected resource/environment. This is metadata only; it does not yet persist connections, resolve secrets or call provider APIs.
- A manual GitHub Actions deployment workflow can deploy a preview Worker or production Worker from main using operator-owned Cloudflare secrets. It does not auto-deploy and does not contain credentials.
- The portable Rust core remains headless. The complete platform distribution is planned to include a browser-based operator console and identity portal; no standalone Otya Android/iOS app, APK or native desktop application is planned.
- Local commands do not make network calls.
- Comprehensive engine gap audit and staged implementation plan documented.
- Obsolete .ot language example removed from the active engine branch.

## Latest provider-contract CI state

At branch head `368588ef52725c9a6edc48cc018331296b57d5a9`, Otya Engine CI [run 46159074134](https://github.com/OtyaHQ/Otya/actions/runs/46159074134) **passed**: rustfmt, Clippy, workspace tests, CLI diagnostics, SQLite smoke tests, example configuration validation, release build and Rust 1.85 minimum-version compile/tests. The portability workflow [run 46159074138](https://github.com/OtyaHQ/Otya/actions/runs/46159074138) has passed Android arm64, armv7, x86_64 and WebAssembly portable-core compilation; the separate Cloudflare Worker packaging/deploy-profile check is still in progress. The new metadata contracts are CI-verified by the core workflow, but they are not a persistent provider registry, vault or live connector.

## CI evidence

- Otya Engine CI [run 265](https://github.com/OtyaHQ/Otya/actions/runs/46143575958), code commit `148f837595e4e42d314fa8b59234fb5363111f01`: **passed** — rustfmt, Clippy, all workspace tests, CLI diagnostics, SQLite `storage init`/`storage check` smoke tests, example configuration validation, release build and Rust 1.85 compile/tests.
- Otya Mobile and Web Portability [run 21](https://github.com/OtyaHQ/Otya/actions/runs/46143575988), same code commit: **passed** — portable core compile checks for Android arm64, Android armv7, Android x86_64 and `wasm32-unknown-unknown`.
- Subsequent commits update documentation; the tested Rust/storage implementation is unchanged from the code commit above.
- The Codespaces devcontainer is committed, but first-launch usability on a real phone browser has not yet been manually validated.

The CI results verify the tested Linux workspace and portable-core target compilation. They do not verify a broad physical Linux distribution/SQLite-version matrix, arbitrary custom install roots, atomic config replacement, general migrations, online backup/restore, crash recovery, real Cloudflare deployment/browser behavior, D1/Queues/Workflows bindings, a first Cloudflare deployment, the future service daemon, provider APIs or production readiness.

## Not yet implemented

- Atomic configuration replacement, backup/restore and schema migrations.
- Crash/power-loss-safe temporary-file replacement for configuration; a process interruption during first creation may leave an incomplete file that must be inspected before retrying.
- Arbitrary operator-selected root paths beyond XDG overrides and validated local instance names.
- Free-space and filesystem-capacity diagnostics.
- General SQLite migration runner beyond the initial v1-to-v2 schema upgrade, module-owned tables, online backup/restore, multi-process stress and crash-recovery support.
- PostgreSQL adapter.
- Per-user service and authenticated local IPC.
- Linux installer, package signatures and service hardening.
- Committed dependency lockfile and CI enforcement of locked dependency resolution.
- Credential vault and secret resolver; persistent provider connection registry; Cloudflare/Firebase/Supabase and generic provider API clients.
- Durable job queue, scheduler, outbox, inbox and webhook verification.
- Backup, restore, migrations and crash recovery.
- Production host-application authorization integration, secure session/token verification and real application-user permissions.
- Async app handlers, request body forwarding, middleware, dynamic route parameters and streaming support.
- Persistent application schemas, migrations, durable operations and background jobs.
- Stable C ABI, Python and optional Kotlin/JNI or Flutter/Dart host-application SDKs; no SDK packages are released yet.
- Android/iOS platform-specific embedding adapters only if a host application needs them; no standalone Otya mobile app or APK pipeline is planned.
- Generic authenticated HTTP management/API server, language-neutral OpenAPI contracts and async remote-storage/operation contracts.
- Browser-based Console, installation-owned Identity portal, Otya UI kit/Studio, provider connection wizard and deployment orchestration.
- Codespaces devcontainer first-launch validation on a real phone browser.
- Cross-platform installers, signed release artifacts, SBOM and build provenance.

## Release status

The branch is an experimental foundation, not a production-ready engine. CI passing is necessary but not sufficient for production readiness. Installation, upgrade, migration, secure configuration lifecycle, backup/restore, fault-injection, security and multi-distribution platform acceptance tests remain outstanding.
