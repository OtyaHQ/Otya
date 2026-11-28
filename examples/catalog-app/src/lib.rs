#![forbid(unsafe_code)]

//! A small stateless example application built on the Otya Engine SDK.
//! Its sample catalogue is demonstration data, not a real store or inventory.

use otya_engine_sdk::{
    AccessPolicy, Application, ApplicationError, ApplicationManifest, ApplicationRequest,
    ApplicationResponse, HttpMethod, RouteDefinition,
};
use serde::Serialize;

static ROUTES: [RouteDefinition; 2] = [
    RouteDefinition::new(HttpMethod::Get, "/items", AccessPolicy::Public, &[]),
    RouteDefinition::new(HttpMethod::Get, "/about", AccessPolicy::Public, &[]),
];

static MANIFEST: ApplicationManifest = ApplicationManifest {
    id: "org.otya.examples.catalog",
    name: "Otya Example Catalog",
    version: "0.1.0",
    engine_api_version: otya_engine_sdk::APPLICATION_API_VERSION,
    base_path: "/api/v1/catalog",
    routes: &ROUTES,
};

#[derive(Serialize)]
struct CatalogResponse {
    application: &'static str,
    demo_only: bool,
    items: &'static [CatalogItem],
}

#[derive(Serialize)]
struct CatalogItem {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    category: &'static str,
}

static ITEMS: [CatalogItem; 3] = [
    CatalogItem {
        id: "starter-kit",
        title: "Starter kit",
        description: "A simple example item served by an Otya-powered API.",
        category: "Examples",
    },
    CatalogItem {
        id: "field-notes",
        title: "Field notes",
        description: "Demonstration catalogue data with no external dependencies.",
        category: "Examples",
    },
    CatalogItem {
        id: "workbench",
        title: "Workbench",
        description: "A sample record showing how an app can own its presentation.",
        category: "Examples",
    },
];

pub struct CatalogApplication;

impl Application for CatalogApplication {
    fn manifest(&self) -> &'static ApplicationManifest {
        &MANIFEST
    }

    fn handle(&self, request: ApplicationRequest) -> Result<ApplicationResponse, ApplicationError> {
        match (request.method(), request.path()) {
            (HttpMethod::Get, "/api/v1/catalog/items") => ApplicationResponse::json(
                200,
                &CatalogResponse {
                    application: "org.otya.examples.catalog",
                    demo_only: true,
                    items: &ITEMS,
                },
            ),
            (HttpMethod::Get, "/api/v1/catalog/about") => ApplicationResponse::json(
                200,
                &serde_json::json!({
                    "application": "Otya Example Catalog",
                    "powered_by": "Otya Engine SDK",
                    "purpose": "Demonstrate an independent application built on the headless engine.",
                    "persistence": "This demonstration uses static sample data only."
                }),
            ),
            _ => Err(ApplicationError::HandlerFailed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otya_engine_sdk::{ApplicationRegistry, ApplicationRequest, CallerContext};

    fn registry() -> ApplicationRegistry {
        let mut registry = ApplicationRegistry::new();
        registry
            .register(CatalogApplication)
            .expect("catalog application should register");
        registry
    }

    #[test]
    fn catalog_endpoint_returns_the_sample_items() {
        let response = registry().dispatch(ApplicationRequest::new(
            HttpMethod::Get,
            "/api/v1/catalog/items",
            Vec::new(),
            CallerContext::anonymous(),
        ));

        assert_eq!(response.status(), 200);
        let value: serde_json::Value =
            serde_json::from_slice(response.body()).expect("catalog response should be JSON");
        assert_eq!(value["demo_only"], true);
        assert_eq!(value["items"].as_array().map(Vec::len), Some(3));
    }

    #[test]
    fn about_endpoint_explains_the_application_boundary() {
        let response = registry().dispatch(ApplicationRequest::new(
            HttpMethod::Get,
            "/api/v1/catalog/about",
            Vec::new(),
            CallerContext::anonymous(),
        ));

        assert_eq!(response.status(), 200);
        let value: serde_json::Value =
            serde_json::from_slice(response.body()).expect("about response should be JSON");
        assert_eq!(value["powered_by"], "Otya Engine SDK");
        assert!(value["persistence"].as_str().is_some());
    }
}
