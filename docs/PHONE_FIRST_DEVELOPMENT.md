# Phone-First Development and Operations

**Goal:** a contributor or operator should be able to work with Otya from a modern phone browser without owning a desktop or laptop. Heavy compilation and tests run on remote compute, not on the phone.

This is a supported development workflow design. It does not mean every editor, keyboard shortcut or terminal experience is equally comfortable on a small screen.

## The three-layer workflow

### 1. Edit from the phone

Open the repository on GitHub in the phone browser.

For lightweight edits, open the repository in [github.dev](https://github.dev). GitHub's web editor can browse and edit the repository and commit source changes without installing a local development toolchain. It has no terminal or build VM, so it cannot run Cargo, tests, or the local Cloudflare development server. Work is stored in browser-local state until it is committed; commit regularly so changes are not trapped in one browser session.

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

Cloudflare is an optional host, not an Otya control plane. The first Rust Worker profile is built on GitHub-hosted runners because the documented default Workers Builds image lists Node.js, Python, PHP, Ruby and Go but does not list Rust. Do not assume the default Cloudflare build image contains a Rust toolchain.

The repository includes a manual GitHub Actions workflow at .github/workflows/cloudflare-worker-deploy.yml. It:
- installs the Rust 1.91 toolchain and WebAssembly target needed by workers-rs 0.8.7;
- builds the headless Worker using worker-build;
- deploys either an isolated preview Worker or the production Worker;
- allows production deployment only from main.

To enable it, the operator stores CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN in GitHub Actions secrets or protected GitHub Environments. The workflow does not include credentials in source control and does not deploy automatically on every push. The workflow file must first be merged into the repository's default branch before it is available for manual workflow_dispatch from the Actions UI.

From a phone, open the repository's Actions tab, select Deploy Otya Cloudflare Worker, choose preview or production, and run the workflow. Production should additionally use a protected GitHub Environment with approval rules. Preview uses a separate Worker name; the current sample exposes only public, non-sensitive health/version endpoints.

For TypeScript/JavaScript Workers, Cloudflare Workers Builds can connect a GitHub repository and build/deploy pushes. For this Rust Worker, use the checked-in GitHub Actions pipeline unless a Rust-capable Workers build image is deliberately configured and tested.

Use one deployment owner per Worker:
- GitHub Actions for the current Rust Worker, because it installs and pins the Rust/WASM toolchain;
- Workers Builds for a separate TypeScript/JavaScript Worker when Cloudflare should own that Worker’s Git-connected build/deploy.

Do not configure both systems to automatically deploy the same Worker from the same production branch. That creates competing deployments and makes release ownership unclear.

For preview safety:
- configure preview-only variables and secrets separately from production;
- bind each preview to separate D1, KV and R2 resources when it needs data isolation;
- never assume a preview is isolated merely because it has a different URL;
- protect preview URLs with Cloudflare Access when they contain non-public application data; preview URLs are public by default;
- do not allow tests or preview jobs to mutate production databases, buckets, queues or live accounts.

Cloudflare preview deployments have resource-isolation limitations. Check the current resource-isolation matrix before enabling each binding.

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
| Deploy the Rust Cloudflare Worker | GitHub Actions workflow_dispatch | GitHub-hosted Rust/WASM runner and Cloudflare account |
| Use an Otya-powered product on a phone | Host application's responsive website or existing mobile client calling Otya HTTPS APIs | Phone browser / host app |
| Operate the headless engine | GitHub Actions and the operator's cloud dashboard | Remote CI/cloud runtime |

## Product boundary

The engine must not depend on GitHub, Codespaces or Cloudflare to run. They are useful developer/operator tools. A customer can instead build locally, use another Git provider, self-host CI, self-host the engine or run it offline.

No Otya account, central Otya build server, mandatory telemetry or vendor heartbeat is introduced by this workflow.


## Engine product boundary

Otya Engine itself is not an APK or mobile application. Android target compilation is a compatibility check for the portable core and a possible future host-application SDK. It does not create an Android app and is not a release objective for this engine repository. User-facing phone experiences belong to the operator's host application or responsive web client.
