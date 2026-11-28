#![forbid(unsafe_code)]

//! Application-facing contracts for building headless services on Otya.
//!
//! The SDK deliberately does not own an HTTP listener, end-user identity
//! system, database connection or provider credential. A trusted host adapter
//! authenticates the incoming caller, constructs CallerContext, and converts
//! the returned response to its transport protocol.

/// Version of the application SDK/host contract.
pub const APPLICATION_API_VERSION: u32 = 1;
/// Maximum request body accepted by the in-process dispatcher (1 MiB).
pub const MAX_APPLICATION_REQUEST_BYTES: usize = 1024 * 1024;
/// Maximum response body returned by an application (2 MiB).
pub const MAX_APPLICATION_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Other,
}

impl HttpMethod {
    pub fn parse(value: &str) -> Self {
        match value {
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "PATCH" => Self::Patch,
            "DELETE" => Self::Delete,
            "OPTIONS" => Self::Options,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessPolicy {
    /// No caller identity is required. The handler must still validate inputs.
    Public,
    /// The host must authenticate the caller before constructing a request.
    Authenticated,
}

/// One exact, versioned route declared by an application manifest.
#[derive(Clone, Copy, Debug)]
pub struct RouteDefinition {
    pub method: HttpMethod,
    /// Relative path beneath the manifest's base_path, such as "/items".
    pub path: &'static str,
    pub access: AccessPolicy,
    /// Every capability is required. Capabilities are granted by the host,
    /// never by a value supplied directly by an untrusted HTTP client.
    pub required_capabilities: &'static [&'static str],
}

impl RouteDefinition {
    pub const fn new(
        method: HttpMethod,
        path: &'static str,
        access: AccessPolicy,
        required_capabilities: &'static [&'static str],
    ) -> Self {
        Self {
            method,
            path,
            access,
            required_capabilities,
        }
    }
}

/// Static metadata and routes for one independently built application.
#[derive(Clone, Copy, Debug)]
pub struct ApplicationManifest {
    /// Reverse-DNS-style application identifier, e.g. "org.example.catalog".
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub engine_api_version: u32,
    /// Canonical prefix such as "/api/v1/catalog".
    pub base_path: &'static str,
    pub routes: &'static [RouteDefinition],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CallerContext {
    subject: Option<String>,
    granted_capabilities: Vec<String>,
}

impl CallerContext {
    pub fn anonymous() -> Self {
        Self::default()
    }

    /// Construct a caller context only after the host has verified the
    /// credential and authorization policy. Never populate it from an
    /// unverified X-User-ID header or a JSON body.
    pub fn from_verified_host<I, S>(
        subject: impl Into<String>,
        granted_capabilities: I,
    ) -> Result<Self, ApplicationError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let subject = subject.into();
        if subject.trim().is_empty()
            || subject.len() > 256
            || subject.chars().any(char::is_control)
        {
            return Err(ApplicationError::InvalidCaller);
        }

        let mut grants = Vec::new();
        for capability in granted_capabilities {
            let capability = capability.into();
            if !valid_capability(&capability) {
                return Err(ApplicationError::InvalidCapability(capability));
            }
            if !grants.contains(&capability) {
                grants.push(capability);
            }
        }

        Ok(Self {
            subject: Some(subject),
            granted_capabilities: grants,
        })
    }

    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    pub fn is_authenticated(&self) -> bool {
        self.subject.is_some()
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.granted_capabilities
            .iter()
            .any(|granted| granted == capability)
    }
}

#[derive(Clone, Debug)]
pub struct ApplicationRequest {
    method: HttpMethod,
    path: String,
    body: Vec<u8>,
    caller: CallerContext,
}

impl ApplicationRequest {
    pub fn new(
        method: HttpMethod,
        path: impl Into<String>,
        body: Vec<u8>,
        caller: CallerContext,
    ) -> Self {
        Self {
            method,
            path: path.into(),
            body,
            caller,
        }
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn caller(&self) -> &CallerContext {
        &self.caller
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationResponse {
    status: u16,
    content_type: String,
    body: Vec<u8>,
}

impl ApplicationResponse {
    pub fn json<T: serde::Serialize>(
        status: u16,
        value: &T,
    ) -> Result<Self, ApplicationError> {
        let body = serde_json::to_vec(value)
            .map_err(|_| ApplicationError::ResponseSerialization)?;
        Ok(Self {
            status,
            content_type: "application/json; charset=utf-8".to_owned(),
            body,
        })
    }

    pub fn text(status: u16, value: impl Into<String>) -> Self {
        Self {
            status,
            content_type: "text/plain; charset=utf-8".to_owned(),
            body: value.into().into_bytes(),
        }
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    fn valid(&self) -> bool {
        (200..=599).contains(&self.status)
            && !self.content_type.is_empty()
            && self.content_type.len() <= 128
            && !self.content_type.chars().any(char::is_control)
            && self.body.len() <= MAX_APPLICATION_RESPONSE_BYTES
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("invalid application manifest: {0}")]
    InvalidManifest(String),
    #[error("the application requires engine API {required}, but this SDK provides {supported}")]
    IncompatibleEngineApi { required: u32, supported: u32 },
    #[error("an application with this ID is already registered")]
    DuplicateApplication,
    #[error("an application route conflicts with an existing route")]
    RouteConflict,
    #[error("caller identity or authorization context is invalid")]
    InvalidCaller,
    #[error("invalid capability identifier: {0}")]
    InvalidCapability(String),
    #[error("application response could not be serialized")]
    ResponseSerialization,
    #[error("application handler failed")]
    HandlerFailed,
}

pub trait Application: Send + Sync {
    fn manifest(&self) -> &'static ApplicationManifest;

    fn handle(
        &self,
        request: ApplicationRequest,
    ) -> Result<ApplicationResponse, ApplicationError>;
}

struct RegisteredApplication {
    application: Box<dyn Application>,
    manifest: &'static ApplicationManifest,
}

/// Small, deterministic exact-path router for in-process application modules.
/// Network listeners, authentication and async host I/O belong to adapters.
#[derive(Default)]
pub struct ApplicationRegistry {
    applications: Vec<RegisteredApplication>,
}

impl ApplicationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<A: Application + 'static>(
        &mut self,
        application: A,
    ) -> Result<(), ApplicationError> {
        let manifest = application.manifest();
        validate_manifest(manifest)?;

        if manifest.engine_api_version != APPLICATION_API_VERSION {
            return Err(ApplicationError::IncompatibleEngineApi {
                required: manifest.engine_api_version,
                supported: APPLICATION_API_VERSION,
            });
        }

        if self
            .applications
            .iter()
            .any(|registered| registered.manifest.id == manifest.id)
        {
            return Err(ApplicationError::DuplicateApplication);
        }

        for new_route in manifest.routes {
            let new_path = full_path(manifest.base_path, new_route.path);
            for registered in &self.applications {
                for existing_route in registered.manifest.routes {
                    if existing_route.method == new_route.method
                        && full_path(registered.manifest.base_path, existing_route.path)
                            == new_path
                    {
                        return Err(ApplicationError::RouteConflict);
                    }
                }
            }
        }

        self.applications.push(RegisteredApplication {
            application: Box::new(application),
            manifest,
        });
        Ok(())
    }

    pub fn dispatch(&self, request: ApplicationRequest) -> ApplicationResponse {
        if request.body.len() > MAX_APPLICATION_REQUEST_BYTES {
            return error_response(413, "request_too_large", "Request body exceeds the 1 MiB limit.");
        }
        if !valid_request_path(&request.path) {
            return error_response(400, "invalid_path", "The request path is invalid.");
        }

        let mut path_exists = false;

        for registered in &self.applications {
            for route in registered.manifest.routes {
                if full_path(registered.manifest.base_path, route.path) != request.path {
                    continue;
                }
                path_exists = true;

                if route.method != request.method {
                    continue;
                }

                if route.access == AccessPolicy::Authenticated
                    && !request.caller.is_authenticated()
                {
                    return error_response(401, "authentication_required", "Authentication is required.");
                }

                if !route
                    .required_capabilities
                    .iter()
                    .all(|required| request.caller.has_capability(required))
                {
                    return error_response(403, "permission_denied", "The caller lacks a required permission.");
                }

                return match registered.application.handle(request) {
                    Ok(response) if response.valid() => response,
                    Ok(_) | Err(_) => {
                        error_response(500, "application_error", "The application could not complete the request.")
                    }
                };
            }
        }

        if path_exists {
            error_response(405, "method_not_allowed", "The HTTP method is not supported for this route.")
        } else {
            error_response(404, "not_found", "No application route matches this request.")
        }
    }

    pub fn application_count(&self) -> usize {
        self.applications.len()
    }
}

fn validate_manifest(manifest: &ApplicationManifest) -> Result<(), ApplicationError> {
    if !valid_application_id(manifest.id) {
        return Err(ApplicationError::InvalidManifest(
            "id must be a reverse-DNS-style lowercase identifier".to_owned(),
        ));
    }
    if manifest.name.trim().is_empty()
        || manifest.name.len() > 128
        || manifest.name.chars().any(char::is_control)
    {
        return Err(ApplicationError::InvalidManifest(
            "name must be 1-128 printable characters".to_owned(),
        ));
    }
    if manifest.version.trim().is_empty()
        || manifest.version.len() > 64
        || !manifest.version.is_ascii()
        || manifest.version.chars().any(char::is_whitespace)
    {
        return Err(ApplicationError::InvalidManifest(
            "version must be a non-empty ASCII version string".to_owned(),
        ));
    }
    if !valid_route_path(manifest.base_path) || manifest.base_path == "/" {
        return Err(ApplicationError::InvalidManifest(
            "base_path must be a canonical non-root absolute path".to_owned(),
        ));
    }
    if manifest.routes.is_empty() {
        return Err(ApplicationError::InvalidManifest(
            "at least one route must be declared".to_owned(),
        ));
    }

    for (index, route) in manifest.routes.iter().enumerate() {
        if !valid_route_path(route.path) {
            return Err(ApplicationError::InvalidManifest(
                "route paths must be canonical absolute paths".to_owned(),
            ));
        }
        if route.required_capabilities.iter().any(|capability| !valid_capability(capability)) {
            return Err(ApplicationError::InvalidManifest(
                "route contains an invalid capability identifier".to_owned(),
            ));
        }
        if manifest.routes[..index].iter().any(|previous| {
            previous.method == route.method && previous.path == route.path
        }) {
            return Err(ApplicationError::InvalidManifest(
                "the manifest contains a duplicate method/path route".to_owned(),
            ));
        }
    }
    Ok(())
}

fn valid_application_id(value: &str) -> bool {
    value.contains('.')
        && value.len() <= 128
        && value.split('.').all(|part| {
            !part.is_empty()
                && part.as_bytes()[0].is_ascii_lowercase()
                && part.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
}

fn valid_capability(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value.split('.').all(|part| {
            !part.is_empty()
                && part.as_bytes()[0].is_ascii_lowercase()
                && part.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
}

fn valid_route_path(path: &str) -> bool {
    if !path.starts_with('/')
        || path.len() > 256
        || path.contains("//")
        || path.contains('?')
        || path.contains('#')
        || path.contains('%')
        || path.chars().any(|character| !character.is_ascii())
    {
        return false;
    }
    if path == "/" {
        return true;
    }
    if path.ends_with('/') {
        return false;
    }

    path[1..].split('/').all(|segment| {
        !segment.is_empty()
            && segment != "."
            && segment != ".."
            && segment.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'.'
            })
    })
}

fn valid_request_path(path: &str) -> bool {
    valid_route_path(path)
}

fn full_path(base_path: &str, route_path: &str) -> String {
    if route_path == "/" {
        base_path.to_owned()
    } else {
        format!("{base_path}{route_path}")
    }
}

fn error_response(status: u16, code: &str, message: &str) -> ApplicationResponse {
    let body = serde_json::json!({
        "error": {
            "code": code,
            "message": message
        }
    });
    ApplicationResponse::json(status, &body).unwrap_or_else(|_| {
        ApplicationResponse::text(500, "Internal Server Error")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    static ROUTES: [RouteDefinition; 2] = [
        RouteDefinition::new(HttpMethod::Get, "/items", AccessPolicy::Public, &[]),
        RouteDefinition::new(
            HttpMethod::Post,
            "/admin/rebuild",
            AccessPolicy::Authenticated,
            &["catalog.index.write"],
        ),
    ];
    static MANIFEST: ApplicationManifest = ApplicationManifest {
        id: "org.example.catalog",
        name: "Example catalog",
        version: "0.1.0",
        engine_api_version: APPLICATION_API_VERSION,
        base_path: "/api/v1/catalog",
        routes: &ROUTES,
    };

    struct TestApplication;

    impl Application for TestApplication {
        fn manifest(&self) -> &'static ApplicationManifest {
            &MANIFEST
        }

        fn handle(
            &self,
            request: ApplicationRequest,
        ) -> Result<ApplicationResponse, ApplicationError> {
            ApplicationResponse::json(200, &serde_json::json!({
                "method": format!("{:?}", request.method()),
                "path": request.path()
            }))
        }
    }

    fn registry() -> ApplicationRegistry {
        let mut registry = ApplicationRegistry::new();
        registry.register(TestApplication).expect("test app should register");
        registry
    }

    fn request(method: HttpMethod, path: &str, caller: CallerContext) -> ApplicationRequest {
        ApplicationRequest::new(method, path, Vec::new(), caller)
    }

    #[test]
    fn registers_and_dispatches_a_public_application_route() {
        let response = registry().dispatch(request(
            HttpMethod::Get,
            "/api/v1/catalog/items",
            CallerContext::anonymous(),
        ));
        assert_eq!(response.status(), 200);
        assert_eq!(response.content_type(), "application/json; charset=utf-8");
    }

    #[test]
    fn rejects_unauthenticated_access_to_protected_routes() {
        let response = registry().dispatch(request(
            HttpMethod::Post,
            "/api/v1/catalog/admin/rebuild",
            CallerContext::anonymous(),
        ));
        assert_eq!(response.status(), 401);
    }

    #[test]
    fn requires_every_declared_capability() {
        let caller = CallerContext::from_verified_host("operator-1", Vec::<String>::new())
            .expect("valid caller");
        let response = registry().dispatch(request(
            HttpMethod::Post,
            "/api/v1/catalog/admin/rebuild",
            caller,
        ));
        assert_eq!(response.status(), 403);
    }

    #[test]
    fn accepts_a_verified_caller_with_the_required_capability() {
        let caller = CallerContext::from_verified_host(
            "operator-1",
            vec!["catalog.index.write".to_owned()],
        )
        .expect("valid caller");
        let response = registry().dispatch(request(
            HttpMethod::Post,
            "/api/v1/catalog/admin/rebuild",
            caller,
        ));
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn returns_method_not_allowed_for_a_known_path() {
        let response = registry().dispatch(request(
            HttpMethod::Delete,
            "/api/v1/catalog/items",
            CallerContext::anonymous(),
        ));
        assert_eq!(response.status(), 405);
    }

    #[test]
    fn returns_not_found_for_unknown_paths() {
        let response = registry().dispatch(request(
            HttpMethod::Get,
            "/api/v1/missing",
            CallerContext::anonymous(),
        ));
        assert_eq!(response.status(), 404);
    }

    #[test]
    fn refuses_duplicate_application_ids() {
        let mut registry = registry();
        assert!(matches!(
            registry.register(TestApplication),
            Err(ApplicationError::DuplicateApplication)
        ));
    }

    #[test]
    fn rejects_path_traversal_and_encoded_paths() {
        let response = registry().dispatch(request(
            HttpMethod::Get,
            "/api/v1/catalog/../items",
            CallerContext::anonymous(),
        ));
        assert_eq!(response.status(), 400);

        let response = registry().dispatch(request(
            HttpMethod::Get,
            "/api/v1/catalog/%2e%2e/items",
            CallerContext::anonymous(),
        ));
        assert_eq!(response.status(), 400);
    }

    #[test]
    fn rejects_oversized_request_bodies() {
        let body = vec![0_u8; MAX_APPLICATION_REQUEST_BYTES + 1];
        let response = ApplicationRegistry::new().dispatch(ApplicationRequest::new(
            HttpMethod::Post,
            "/api/v1/catalog/items",
            body,
            CallerContext::anonymous(),
        ));
        assert_eq!(response.status(), 413);
    }

    #[test]
    fn caller_context_rejects_control_characters() {
        assert!(CallerContext::from_verified_host("operator\nforged", Vec::<String>::new()).is_err());
    }
}
