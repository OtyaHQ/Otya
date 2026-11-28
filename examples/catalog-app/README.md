# Otya Example Catalog Application

This is a small, independent application built using the Otya Engine SDK. It is deliberately separate from the engine core.

## What it demonstrates

- An application manifest with a stable reverse-DNS ID and engine API version.
- Exact, versioned HTTP route declarations.
- Public versus authenticated route policy and required capability declarations.
- A handler returning a JSON response through the application SDK.
- A separate website in `platforms/cloudflare-worker/public/index.html` that calls the application's API.

## API

- `GET /api/v1/catalog/items`
- `GET /api/v1/catalog/about`

The records are static demonstration data. This example does not provide persistence, user accounts, order processing, payments, a job queue or production authentication.

## Test the application contract

From the repository root:

    cargo test -p otya-engine-sdk -p otya-example-catalog-app

The Cloudflare Worker adapter imports this application and routes API requests through the same SDK. The Worker serves the responsive website from Cloudflare Static Assets.

For the architecture and how to create a separate application, see [Applications on Otya](../../docs/APPLICATIONS_ON_OTYA.md).
