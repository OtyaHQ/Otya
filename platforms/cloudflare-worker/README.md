# Otya Cloudflare Worker Host Adapter

**Status:** headless Rust/WebAssembly host adapter; CI compile verification and first deployment are pending.

This project is a separate Cloudflare Worker wrapper around the portable Otya Rust core. It is not an APK, Android application, desktop application, Linux daemon or shared Otya-hosted control plane.

## Current routes and website

- GET /health — non-sensitive Worker health response.
- GET /version — adapter package version.
- GET /capabilities — non-sensitive runtime and application API compatibility metadata, including implemented capabilities and explicit current limitations.
- GET /api/v1/catalog/items — example application API, returning static demonstration records.
- GET /api/v1/catalog/about — example application metadata.
- Other `/api/*` paths are dispatched through the SDK registry and return standard 404/405 responses.
- Other paths are served through the Workers Static Assets binding from `public/index.html` and its assets.

The website calls the catalog API using same-origin HTTPS. The example app has no database, user accounts, orders, payments or live inventory. Its routes are public and read-only. This is a proof of the application/frontend boundary, not a production backend.

## Why this is a separate Cargo workspace

The main Otya core keeps its Rust 1.85 minimum. The workers-rs 0.8.7 host adapter requires Rust 1.91, so this small platform package is intentionally isolated from the root Cargo workspace. It depends on the shared core by path and does not raise the main engine's toolchain requirement.

## Build checks

From the repository root:

    rustup toolchain install 1.91.0 --profile minimal
    rustup target add wasm32-unknown-unknown --toolchain 1.91.0
    cargo +1.91.0 fmt --manifest-path platforms/cloudflare-worker/Cargo.toml -- --check
    cargo +1.91.0 check --manifest-path platforms/cloudflare-worker/Cargo.toml --target wasm32-unknown-unknown

The root rust-toolchain.toml selects the engine's stable toolchain, so use the explicit +1.91.0 toolchain selector for this adapter.

The Wrangler build configuration invokes worker-build --release. The deployment workflow installs worker-build 0.8.7 and Wrangler 4.169.0 in a GitHub-hosted runner.

## Deploy from a phone

The manual deployment workflow is .github/workflows/cloudflare-worker-deploy.yml. It becomes available in the GitHub Actions UI after the workflow is present on the repository's default branch.

1. In Cloudflare, choose the account that will own the Worker.
2. Create an API token with only the Worker deployment permissions required for that account.
3. In GitHub repository Settings, add Actions secrets named CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN. Store the values as secrets; never commit them.
4. Open Actions → Deploy Otya Cloudflare Worker.
5. Choose preview to publish a separate otya-engine-edge-preview Worker, or production to publish otya-engine-edge.
6. Production is allowed only from main. Configure the GitHub production Environment with required reviewers before using production deployments.

The preview and production Worker names are separate. The health/version/capabilities endpoints disclose no credentials or private application data. The capabilities document is descriptive metadata, not an authorization grant. Future protected endpoints and data bindings must be explicitly authenticated and isolated.

## Application and security boundary

The Worker registers the sample catalog app through `otya-engine-sdk`. Its public route list is intentionally read-only. The current adapter constructs an anonymous caller context; no protected application route should be exposed until a real host session/token verifier and authorization mapping create a verified `CallerContext`. Never trust an end-user identity or capability read directly from an arbitrary request header or body.

The current Worker adapter does not forward request bodies into the SDK, and the SDK registry is synchronous and exact-route-only. Dynamic path parameters, middleware, authentication/session verification, persistent storage and async job execution are future work.

## Runtime boundary

Cloudflare Workers run WebAssembly in the Workers runtime. This adapter cannot assume native Linux SQLite, arbitrary local disk, process spawning or a continuously running daemon. Persistent state, queues and durable orchestration require separate host-specific adapters.

The synchronous core Storage contract currently used by native SQLite is not suitable as a direct interface to asynchronous D1/Queues/Workflows bindings. An asynchronous host/storage contract and conformance tests must be designed before those services are connected.

For full Linux processes and system packages, use a separately tested Linux container or VPS profile instead of pretending the Worker is a general Linux machine. See docs/HEADLESS_AND_PORTABLE_RUNTIME.md.
