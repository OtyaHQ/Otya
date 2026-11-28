use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A provider family supported by a first-party or generic connector.
///
/// This is metadata only. It does not create an account, make a network call,
/// store a credential value, or imply that every provider capability is ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Cloudflare,
    Firebase,
    Supabase,
    Smtp,
    Resend,
    GenericHttp,
}

/// Authentication mechanism used by a provider connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAuthMethod {
    OAuth2,
    ApiToken,
    ServiceAccount,
    MutualTls,
    /// For provider configuration deliberately designed to be public, such as
    /// a Firebase client API key. This is not appropriate for privileged keys.
    PublicConfiguration,
    None,
}

/// Where a credential is stored. The connection model stores only a reference
/// to the secret; the secret value is owned by a separate vault/runtime adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStore {
    OperatingSystem,
    EncryptedLocalVault,
    OperatorManagedVault,
    ProviderNativeStore,
}

/// A non-secret pointer to a credential in an operator-controlled store.
///
/// There is intentionally no field for a password, API token, private key or
/// other credential value. Implementations must resolve this reference only
/// inside a trusted backend process and must never return the resolved value
/// to a browser, manifest, log or diagnostic response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialReference {
    /// Operator-local identifier for the secret entry.
    pub reference: String,
    pub store: CredentialStore,
    /// Optional opaque version/rotation identifier.
    pub version: Option<String>,
}

impl CredentialReference {
    pub fn validate(&self) -> Result<(), ProviderModelError> {
        validate_identifier("credential reference", &self.reference)?;
        if let Some(version) = &self.version {
            validate_label("credential version", version)?;
        }
        Ok(())
    }
}

/// An operator-owned provider account connection.
///
/// This record contains identity and permission metadata only. It must never
/// contain raw API keys, OAuth tokens, service-account JSON or private keys.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderConnection {
    pub id: String,
    pub provider: ProviderKind,
    pub display_name: String,
    pub auth_method: ProviderAuthMethod,
    #[serde(default)]
    pub credential_references: Vec<CredentialReference>,
    /// Provider-native permission/scope names granted to this connection.
    #[serde(default)]
    pub granted_scopes: Vec<String>,
}

impl ProviderConnection {
    pub fn validate(&self) -> Result<(), ProviderModelError> {
        validate_identifier("connection ID", &self.id)?;
        validate_label("connection display name", &self.display_name)?;

        let requires_credential = !matches!(
            self.auth_method,
            ProviderAuthMethod::None | ProviderAuthMethod::PublicConfiguration
        );

        if requires_credential && self.credential_references.is_empty() {
            return Err(ProviderModelError::MissingCredentialReference);
        }

        if matches!(
            self.auth_method,
            ProviderAuthMethod::None | ProviderAuthMethod::PublicConfiguration
        ) && !self.credential_references.is_empty()
        {
            return Err(ProviderModelError::UnexpectedCredentialReference);
        }

        for (index, credential) in self.credential_references.iter().enumerate() {
            credential.validate()?;
            if self.credential_references[..index]
                .iter()
                .any(|existing| existing.reference == credential.reference)
            {
                return Err(ProviderModelError::DuplicateCredentialReference);
            }
        }

        validate_unique_labels("provider scope", &self.granted_scopes)?;
        Ok(())
    }
}

/// Environment scope for a resource binding or project grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectEnvironment {
    Development,
    Staging,
    Production,
}

/// A provider resource selected by the operator for one application project.
///
/// A connection can represent one provider account; bindings select individual
/// resources within that account and keep environment boundaries explicit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderResourceBinding {
    pub id: String,
    pub connection_id: String,
    pub project_id: String,
    pub environment: ProjectEnvironment,
    /// Provider-specific resource type, e.g. "workers_script", "d1_database".
    pub resource_type: String,
    /// Opaque provider resource identifier. Do not put credentials here.
    pub resource_id: String,
    #[serde(default)]
    pub available_capabilities: Vec<String>,
}

impl ProviderResourceBinding {
    pub fn validate(&self) -> Result<(), ProviderModelError> {
        validate_identifier("resource binding ID", &self.id)?;
        validate_identifier("connection ID", &self.connection_id)?;
        validate_identifier("project ID", &self.project_id)?;
        validate_label("resource type", &self.resource_type)?;
        validate_label("resource ID", &self.resource_id)?;
        validate_unique_labels("resource capability", &self.available_capabilities)
    }
}

/// Explicit permission for a project/environment to use one provider resource.
///
/// This grant is not an application user's role and does not authorize
/// end-user business actions. It is an operator-level service capability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderProjectGrant {
    pub id: String,
    pub project_id: String,
    pub connection_id: String,
    pub resource_binding_id: String,
    pub environment: ProjectEnvironment,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl ProviderProjectGrant {
    pub fn validate(
        &self,
        binding: &ProviderResourceBinding,
    ) -> Result<(), ProviderModelError> {
        validate_identifier("grant ID", &self.id)?;
        validate_identifier("project ID", &self.project_id)?;
        validate_identifier("connection ID", &self.connection_id)?;
        validate_identifier("resource binding ID", &self.resource_binding_id)?;

        if self.project_id != binding.project_id
            || self.connection_id != binding.connection_id
            || self.resource_binding_id != binding.id
            || self.environment != binding.environment
        {
            return Err(ProviderModelError::GrantBindingMismatch);
        }

        validate_unique_labels("granted capability", &self.capabilities)?;
        if self.capabilities.is_empty() {
            return Err(ProviderModelError::EmptyCapabilityGrant);
        }

        for capability in &self.capabilities {
            if !binding.available_capabilities.contains(capability) {
                return Err(ProviderModelError::CapabilityNotAvailable(
                    capability.clone(),
                ));
            }
        }

        Ok(())
    }
}

fn validate_identifier(field: &'static str, value: &str) -> Result<(), ProviderModelError> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if valid {
        Ok(())
    } else {
        Err(ProviderModelError::InvalidIdentifier(field))
    }
}

fn validate_label(field: &'static str, value: &str) -> Result<(), ProviderModelError> {
    if value.trim().is_empty()
        || value.len() > 256
        || value.chars().any(char::is_control)
    {
        Err(ProviderModelError::InvalidLabel(field))
    } else {
        Ok(())
    }
}

fn validate_unique_labels(
    field: &'static str,
    values: &[String],
) -> Result<(), ProviderModelError> {
    for (index, value) in values.iter().enumerate() {
        validate_label(field, value)?;
        if values[..index].contains(value) {
            return Err(ProviderModelError::DuplicateLabel(field));
        }
    }
    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProviderModelError {
    #[error("invalid {0}; use 1-64 ASCII letters, digits, dots, underscores or hyphens")]
    InvalidIdentifier(&'static str),
    #[error("{0} must be non-empty, at most 256 bytes and contain no control characters")]
    InvalidLabel(&'static str),
    #[error("provider connection requires at least one credential reference")]
    MissingCredentialReference,
    #[error("this authentication method must not have credential references")]
    UnexpectedCredentialReference,
    #[error("provider connection contains a duplicate credential reference")]
    DuplicateCredentialReference,
    #[error("{0} contains a duplicate or invalid value")]
    DuplicateLabel(&'static str),
    #[error("provider project grant does not match its resource binding")]
    GrantBindingMismatch,
    #[error("provider project grant must include at least one capability")]
    EmptyCapabilityGrant,
    #[error("provider resource does not expose requested capability: {0}")]
    CapabilityNotAvailable(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> ProviderConnection {
        ProviderConnection {
            id: "cloudflare-company".to_owned(),
            provider: ProviderKind::Cloudflare,
            display_name: "Company Cloudflare".to_owned(),
            auth_method: ProviderAuthMethod::ApiToken,
            credential_references: vec![CredentialReference {
                reference: "vault-cloudflare-deploy".to_owned(),
                store: CredentialStore::EncryptedLocalVault,
                version: Some("v1".to_owned()),
            }],
            granted_scopes: vec![
                "workers_scripts_write".to_owned(),
                "workers_routes_write".to_owned(),
            ],
        }
    }

    fn binding() -> ProviderResourceBinding {
        ProviderResourceBinding {
            id: "opoint-api-worker".to_owned(),
            connection_id: "cloudflare-company".to_owned(),
            project_id: "opoint".to_owned(),
            environment: ProjectEnvironment::Staging,
            resource_type: "workers_script".to_owned(),
            resource_id: "opoint-api-staging".to_owned(),
            available_capabilities: vec![
                "deploy".to_owned(),
                "read_logs".to_owned(),
            ],
        }
    }

    #[test]
    fn accepts_a_connection_with_a_secret_reference() {
        assert!(connection().validate().is_ok());
    }

    #[test]
    fn rejects_api_tokens_without_a_secret_reference() {
        let mut value = connection();
        value.credential_references.clear();
        assert_eq!(
            value.validate(),
            Err(ProviderModelError::MissingCredentialReference)
        );
    }

    #[test]
    fn rejects_credentials_for_public_configuration_auth() {
        let mut value = connection();
        value.auth_method = ProviderAuthMethod::PublicConfiguration;
        assert_eq!(
            value.validate(),
            Err(ProviderModelError::UnexpectedCredentialReference)
        );
    }

    #[test]
    fn rejects_duplicate_scopes() {
        let mut value = connection();
        value.granted_scopes.push("workers_scripts_write".to_owned());
        assert_eq!(
            value.validate(),
            Err(ProviderModelError::DuplicateLabel("provider scope"))
        );
    }

    #[test]
    fn a_project_grant_must_match_its_resource_environment() {
        let mut grant = ProviderProjectGrant {
            id: "grant-opoint-staging".to_owned(),
            project_id: "opoint".to_owned(),
            connection_id: "cloudflare-company".to_owned(),
            resource_binding_id: "opoint-api-worker".to_owned(),
            environment: ProjectEnvironment::Staging,
            capabilities: vec!["deploy".to_owned()],
        };
        assert!(grant.validate(&binding()).is_ok());

        grant.environment = ProjectEnvironment::Production;
        assert_eq!(
            grant.validate(&binding()),
            Err(ProviderModelError::GrantBindingMismatch)
        );
    }

    #[test]
    fn a_project_cannot_be_granted_a_capability_the_resource_does_not_have() {
        let grant = ProviderProjectGrant {
            id: "grant-opoint-staging".to_owned(),
            project_id: "opoint".to_owned(),
            connection_id: "cloudflare-company".to_owned(),
            resource_binding_id: "opoint-api-worker".to_owned(),
            environment: ProjectEnvironment::Staging,
            capabilities: vec!["delete_production_database".to_owned()],
        };
        assert_eq!(
            grant.validate(&binding()),
            Err(ProviderModelError::CapabilityNotAvailable(
                "delete_production_database".to_owned()
            ))
        );
    }

    #[test]
    fn provider_metadata_serializes_without_raw_credential_values() {
        let encoded = toml::to_string(&connection()).expect("metadata serializes");
        assert!(encoded.contains("vault-cloudflare-deploy"));
        assert!(!encoded.contains("secret_value"));
        assert!(!encoded.contains("api_key"));
    }
}
