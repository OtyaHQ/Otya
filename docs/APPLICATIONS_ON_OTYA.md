# Applications and Websites on Otya Engine

## Product rule

**Otya Engine is the foundation; applications are built on top of it.**

The engine is not one giant business application and does not require every developer to use an Otya interface. A developer creates an independent application, declares its application contract and capabilities, and uses Otya APIs and modules to avoid rebuilding common infrastructure.

The application owns its business model, product interface, resource permissions and business authorization. It may use the Engine installation's private identity provider for sign-in, or keep a separate product identity system. Otya supplies reusable runtime, operator UI, identity, UI-kit and infrastructure capabilities. Each operator controls the environment and provider accounts used by that application.

The portable Rust core is headless. The complete installation includes a browser-based operator console and may include reusable UI components, templates and schema-driven app-building tools. It is not an APK, Android/iOS app or app-store product. See [Private platform distribution](PRIVATE_PLATFORM_DISTRIBUTION.md).

## Independent operator model

Otya is installed separately by each developer or organization. Their application source remains in their own repository; the Engine is a dependency and runtime underneath it. An operator can run several applications inside one intentionally shared trust boundary or create separate named installations for stronger isolation. Each installation may have its own user directory, login portal and shared services; unrelated installations do not share them. The Engine does not provide a cross-customer multi-tenant pool. See [Building Independent Applications on Otya](INDEPENDENT_OPERATOR_GUIDE.md) for executable current setup steps, the actual shipped surface, non-Rust limitations and product/Worker retirement policy.

## What an Otya-powered product looks like

```text
Phone / desktop / browser clients
          |
          | HTTPS, JSON, signed webhooks
          v
Application website or mobile frontend
          |
          v
Host/runtime adapter (Worker or Linux service)
          |
          v
Otya application SDK and route registry
          |
          +--> Application-owned domain logic
          |
          +--> Otya modules and capability contracts
          |
          v
Portable Otya Engine core
          |
          +--> Storage and transactions
          +--> Durable operations and events
          +--> Security and audit
          +--> Provider adapters
          |
          v
Operator-selected host and infrastructure
```

This is a logical model. Not all of the listed capabilities are implemented yet. In particular, the current example has no durable store, background job engine or general authentication adapter.

## Three ways to use the engine

### 1. Embed the Rust library

A Rust application can link Otya as a dependency and use its public contracts in the same process. The host application owns the process lifecycle and UI. The first application SDK is `otya-engine-sdk`; it is currently a local workspace crate with `publish = false`, not yet a released crates.io package.

This is the most direct mode for a Rust application.

### 2. Connect to a headless Otya service

A non-Rust application can connect to an operator-managed HTTPS API or an optional local IPC client once those host adapters are implemented. A website built in TypeScript, Python service, Flutter application or other frontend must not need to link Rust directly.

The generic authenticated HTTP service and stable public management API are still roadmap work. Do not assume that every non-Rust application can connect to a production service today.

### 3. Deploy an application with a cloud host adapter

The initial Cloudflare adapter compiles the portable core and application SDK to WebAssembly. It can serve a static website and dispatch API routes in the same Worker. A different host adapter will be required for a full Linux process or another cloud provider.

Cloudflare is optional and operator-controlled. GitHub is used here for source control, remote builds and the manual deployment workflow; neither GitHub nor Cloudflare is a dependency of the portable core.

## Application manifest and route contract

Every application declares:

- a stable reverse-DNS-style application ID;
- name and application version;
- compatible Otya application API version;
- a canonical base path;
- exact HTTP method/path routes;
- whether each route is public or requires an authenticated caller;
- required capability grants for protected routes.

The manifest is code-defined in the first SDK iteration. A future signed package manifest will need schema versioning, dependency resolution, signature verification, upgrade rules and a safe disable/uninstall lifecycle.

The initial SDK's `ApplicationRegistry` rejects duplicate application IDs, incompatible API versions, route collisions, invalid route paths, path traversal and oversized requests. It enforces route authentication and all required capability grants before invoking the handler.

The registry is synchronous and in-process. It is not yet an asynchronous web framework, plugin loader, dynamic route-parameter router or production middleware system.

## Authentication and authorization boundary

The engine must not trust an arbitrary caller-supplied user ID, tenant ID, role or capability.

The host application or trusted runtime adapter must:
1. verify the session/token/certificate using the application's actual identity system;
2. authorize the requested operation;
3. construct a `CallerContext` from the verified subject and scoped capability grants;
4. pass that context to the SDK.

`CallerContext::from_verified_host` documents this trust boundary, but is not itself a token verifier. Do not build a route that grants privileges based on an unverified `X-User-ID`, `X-Role`, URL parameter or request body.

The current Cloudflare sample exposes only public, read-only demonstration routes. It passes an anonymous caller intentionally. No protected business route should be deployed until a real verifier and host authorization adapter are implemented and tested.

## Website and API together

A website is a frontend that can be deployed separately from the engine. It calls the application's API over HTTPS. The static assets can be served by a CDN, object host or a runtime's static asset feature.

The first Cloudflare example uses Workers Static Assets:
- the HTML/CSS/JavaScript website is under `platforms/cloudflare-worker/public/`;
- the Worker handles `/health`, `/version` and `/capabilities`;
- Worker-first routing sends `/api/*` to the application registry;
- remaining paths are served by the static-assets binding.

The sample page calls `GET /api/v1/catalog/items` and renders the JSON response. It demonstrates the boundary between an application frontend and an Otya-powered API.

The `/capabilities` document reports the Engine version, application API version, host runtime, implemented features and current limitations. It is descriptive only; it does not authorize requests or replace host authentication.

The same pattern can power a responsive web application or an existing phone app's API calls. The engine does not need to become an Android app. Frontends must not contain provider API tokens or server-side credentials.

For a Linux deployment, the same application contract will be exposed through a future native HTTP host adapter behind a reverse proxy such as Caddy or Nginx. That adapter is not implemented yet.

## Current example

`examples/catalog-app` is a small stateless application built on the SDK.

- `GET /api/v1/catalog/items` returns three sample records.
- `GET /api/v1/catalog/about` explains the demonstration.
- The sample website calls the first endpoint.
- Tests exercise the registry and JSON application routes.

These records are static fixtures. This is not a real catalogue database, shop, inventory system, payment flow or production commerce service.

Run the application contract tests from a Rust development environment:

```sh
cargo test -p otya-engine-sdk -p otya-example-catalog-app
```

Run the full native workspace checks:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

The Worker is a separate Cargo workspace because it uses a newer Rust/Workers toolchain. Its WebAssembly build check is in the Otya Portable Core Targets workflow.

## How developers will build real applications

The intended developer workflow is:

1. Add Otya's published SDK or a host-specific client to the application.
2. Declare the application manifest, API version, routes and requested capabilities.
3. Implement the application's own domain logic and user-facing frontend.
4. Select the storage and provider adapters the application actually needs.
5. Configure the runtime adapter (native Linux, Worker or another supported host).
6. Run contract tests, security checks and adapter conformance tests.
7. Deploy through the operator's own infrastructure and credentials.
8. Upgrade the application and its Otya modules using compatibility checks and reversible migrations.

The CLI's future project scaffolder should generate a starter app, manifest, routes, tests, deployment profile and example frontend. A scaffolder must not silently register a cloud account, create a tenant, provision billable provider resources or deploy to production.

## Not implemented yet

- Production-ready application package/manifest loader and signed module distribution.
- Async application handlers and stable async host contracts.
- Authentication/session verification and host authorization adapters.
- General HTTPS server for Linux and other non-Cloudflare hosts.
- Route parameters, middleware, OpenAPI generation, request schema validation and streaming body handling.
- Durable application storage, business schemas, migrations, backup and restore.
- Jobs, scheduling, queue/outbox/inbox, webhook delivery and reconciliation.
- Generic multi-app supervisor, application isolation and per-app resource limits.
- Published SDK release, semver compatibility policy and templates/scaffolder.
- First deployed Worker preview and real browser end-to-end test.

The SDK and sample are a foundation to verify the architecture, not a production application platform yet.
