# Mobile Runtime Model

## Otya Engine is not a mobile app

Otya Engine is a headless systems engine. This repository does not aim to ship an Otya Android APK, iOS application, desktop GUI or app-store product.

A phone has two roles:

1. **Development and operations device:** use GitHub in a mobile browser to edit source, review pull requests, start remote GitHub Actions workflows and inspect build/deployment logs. Codespaces provides remote compute and a terminal when needed.
2. **End-user device:** use the host application's existing responsive website or mobile app to call the operator's Otya HTTPS API.

The engine runs on a Linux host, cloud runtime, Worker or container chosen by the operator. It does not have to run inside the phone or be installed as a mobile app. Its services continue running when the phone closes the browser or loses connectivity.

## Optional embedding is not a product requirement

The portable core is compile-checked for selected Android targets. That check preserves the option for a host application developer to link a native Otya library in a future integration. It does not mean the engine is an Android app, that an APK is being built, or that a mobile package is released.

A future Kotlin/JNI or Flutter/Dart package would be an optional SDK for developers embedding Otya inside their own host applications. It would not be a standalone Otya app and would not replace the headless server runtime. Do not begin that work until the versioned C ABI, ownership, lifecycle, permissions and compatibility contracts are stable.

## Phone-to-engine interaction

Phone browsers and host mobile applications should use:

- HTTPS APIs with explicit authentication and authorization;
- short-lived or scoped credentials;
- versioned JSON request and response contracts;
- signed webhooks for inbound provider events;
- bounded request sizes, timeouts and rate limits;
- idempotency keys for operations that may be retried;
- health and readiness endpoints that expose no secrets or private configuration.

Do not put provider API tokens in mobile application bundles or browser JavaScript. The host application remains responsible for its end users and permissions. The Otya engine remains independently operated, with no mandatory Otya login, central user directory or heartbeat.

## No-PC development workflow

The code editor may run in GitHub's browser or a Codespace. Compilation, tests and builds run in GitHub Actions. The Cloudflare deployment workflow is manually started from the GitHub Actions UI after it exists on the repository's default branch. The deployment account credentials live in GitHub Actions secrets, not on the phone and not in source control.

See [Phone-First Development](PHONE_FIRST_DEVELOPMENT.md) and [Headless and Portable Runtime](HEADLESS_AND_PORTABLE_RUNTIME.md).
