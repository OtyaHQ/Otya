# Phone-First Development and Operations

**Goal:** a contributor or operator should be able to work with Otya from a modern phone browser without owning a desktop or laptop. Heavy compilation and tests run on remote compute, not on the phone.

This is a supported development workflow design. It does not mean every editor, keyboard shortcut or terminal experience is equally comfortable on a small screen.

## The three-layer workflow

### 1. Edit from the phone

Open the repository on GitHub in the phone browser.

For lightweight edits, open the repository in [github.dev](https://github.dev). GitHub's web editor can browse and edit the repository and commit source changes without installing a local development toolchain. It has no terminal or build VM, so it cannot run Cargo, tests, Android builds or the local Cloudflare development server. Work is stored in browser-local state until it is committed; commit regularly so changes are not trapped in one browser session.

For the full development environment, create or resume a GitHub Codespace from the repository's **Code → Codespaces** menu. Open the Codespace in the browser editor. The Codespace is a cloud VM with a terminal and the repository's `.devcontainer` environment.

The repository's `.devcontainer` installs:
- Rust stable, rustfmt and Clippy;
- system SQLite development headers and pkg-config for the optional Linux adapter;
- Rust standard-library targets for Android arm64, Android x86_64, Android armv7 and WebAssembly;
- Rust Analyzer and TOML editor support.

The environment runs the workspace tests after creation. Once it is ready, the phone can be used to edit, run commands, inspect test output, create branches, commit and push changes. No local PC compilation is required.

Typical commands in a Codespace:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

GitHub documents the browser editor as a lightweight editor without associated compute or an integrated terminal; Codespaces provides the VM and terminal. Mobile browser layouts have smaller-screen limitations. Codespaces usage is subject to the user's account quota and billing settings.

### 2. Let CI perform the heavy work

After pushing a branch or opening a pull request, GitHub Actions runs in the cloud. The Otya Engine workflows are responsible for repeatable formatting, linting, tests, release builds and platform compile checks.

The mobile portability workflow checks that the platform-neutral Rust core compiles for:
- Android arm64;
- Android x86_64;
- Android armv7;
- WebAssembly `wasm32-unknown-unknown`.

These are compile checks only. They do not yet create an Android APK, iOS application, native FFI bridge, emulator test or published mobile app.

The phone may be switched off after the change is pushed: GitHub Actions continues on GitHub-hosted runners. The result, logs and artifacts are viewed later from GitHub in the phone browser.

### 3. Deploy through the operator's cloud account

Cloudflare remains an optional host and provider, not a required Otya Engine control plane.

For a Cloudflare Worker project, an operator can connect its GitHub repository through **Workers Builds**. Cloudflare can build pushes to the chosen production branch and create branch/PR Previews. Keep `main` as the production branch; feature branches and pull requests should use previews.

Use one deployment owner per Worker:
- **Workers Builds** when Cloudflare should own the Git-connected build and deploy for that Worker; or
- **GitHub Actions** when the project needs one external pipeline to build and deploy multiple artifact types or providers.

Do not configure both systems to automatically deploy the same Worker from the same production branch. That creates competing deployments and makes release ownership unclear.

For preview safety:
- configure preview-only variables and secrets separately from production;
- bind each preview to separate D1, KV and R2 resources when it needs data isolation;
- never assume a preview is isolated merely because it has a different URL;
- protect preview URLs with Cloudflare Access when they contain non-public application data; preview URLs are public by default;
- do not allow tests or preview jobs to mutate production databases, buckets, queues or live accounts.

Cloudflare Previews have specific product limitations: some resources are shared unless separately bound, and certain routes, cron triggers, queue consumers and workflow bindings may still use production resources. Check the current resource-isolation matrix before enabling each binding.

## Secrets and account security

A phone browser is a control surface, not a place to store deployment credentials.

- Keep provider tokens out of source files, commits, screenshots, chat logs and app bundles.
- Use GitHub Actions secrets and protected GitHub Environments for deployment credentials.
- Give each CI integration only the permissions it needs.
- Require review/approval for production deployment where the repository and plan support it.
- Use separate credentials and resources for test, preview and production.
- Do not expose Cloudflare API tokens or GitHub write tokens to an end-user mobile app.
- Prefer short-lived, scoped credentials and rotate any token that may have been exposed.

## What can be done entirely from a phone?

| Work | Phone-only path | What does the work |
|---|---|---|
| Browse and edit small files | GitHub web editor / github.dev | Phone browser |
| Run Rust commands and tests | Codespaces in browser | Remote Codespace VM |
| Run formatting, Clippy and CI | GitHub Actions | GitHub-hosted runner |
| Compile the portable core for Android/WASM | Mobile portability workflow | GitHub-hosted runner |
| Review pull requests and logs | GitHub website | Phone browser |
| Build and preview a Cloudflare Worker | Workers Builds / Previews | Cloudflare build infrastructure |
| Use an Otya-powered mobile app | Installed Android app or responsive web app | Phone |
| Build/publish an Android app | Future protected mobile release workflow | Remote Android build runner |

## Product boundary

The engine must not depend on GitHub, Codespaces or Cloudflare to run. They are useful developer/operator tools. A customer can instead build locally, use another Git provider, self-host CI, self-host the engine or run it offline.

No Otya account, central Otya build server, mandatory telemetry or vendor heartbeat is introduced by this workflow.
