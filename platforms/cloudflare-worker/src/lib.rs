#![forbid(unsafe_code)]

//! Headless Cloudflare Workers host for applications built with Otya Engine.
//!
//! Static website assets are an optional host concern. API requests are
//! dispatched through the same application SDK used by native applications.
//! This adapter does not add a central Otya account or customer directory.

use otya_engine_sdk::{
    ApplicationRegistry, ApplicationRequest, CallerContext, HttpMethod,
};
use otya_example_catalog_app::CatalogApplication;
use worker::{Context, Env, Headers, Method, Request, Response, Result, event};

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();

    if path == "/health" {
        return if req.method() == Method::Get {
            Response::ok("ok\nruntime=cloudflare-workers-wasm\n")
        } else {
            Response::error("Method Not Allowed", 405)
        };
    }

    if path == "/version" {
        return if req.method() == Method::Get {
            Response::ok(env!("CARGO_PKG_VERSION"))
        } else {
            Response::error("Method Not Allowed", 405)
        };
    }

    if path.starts_with("/api/") {
        let mut registry = ApplicationRegistry::new();
        registry
            .register(CatalogApplication)
            .map_err(|_| worker::Error::RustError("application registry configuration is invalid".into()))?;

        // Authentication is intentionally anonymous in this public read-only
        // sample. Protected routes must receive a CallerContext created by an
        // adapter only after it verifies the host application's credentials.
        let request = ApplicationRequest::new(
            HttpMethod::parse(req.method().as_ref()),
            path,
            Vec::new(),
            CallerContext::anonymous(),
        );
        let response = registry.dispatch(request);

        let headers = Headers::new();
        headers.set("content-type", response.content_type())?;
        headers.set("cache-control", "no-store")?;
        headers.set("x-content-type-options", "nosniff")?;
        return Response::from_bytes(response.body().to_vec())
            .map(|body| body.with_status(response.status()).with_headers(headers));
    }

    // Everything outside health/version/API is served by the optional static
    // asset binding configured in wrangler.toml.
    env.assets("ASSETS")?.fetch_request(req).await
}
