#![forbid(unsafe_code)]

//! Headless Cloudflare Workers entrypoint.
//!
//! The adapter deliberately contains no business state, local filesystem,
//! SQLite, company registry or Otya-hosted control-plane behavior. It calls
//! into the shared portable core and exposes only non-sensitive health/version
//! metadata. Provider bindings and asynchronous storage adapters are added
//! separately after their contracts pass conformance tests.

use otya_engine_core::CONFIG_SCHEMA_VERSION;
use worker::{Context, Env, Method, Request, Response, Result, event};

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    if req.method() != Method::Get {
        return Response::error("Method Not Allowed", 405);
    }

    match req.path().as_str() {
        "/health" => Response::ok(format!(
            "ok\nruntime=cloudflare-workers-wasm\nconfig_schema_version={CONFIG_SCHEMA_VERSION}\n"
        )),
        "/version" => Response::ok(env!("CARGO_PKG_VERSION")),
        _ => Response::error("Not Found", 404),
    }
}
