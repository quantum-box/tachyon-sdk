use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_pool: Option<AuthUserPool>,
    #[serde(default)]
    pub providers: Vec<AuthProvider>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthUserPool {
    Shared,
    Dedicated,
}

impl AuthUserPool {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::Dedicated => "dedicated",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthProvider {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: AuthProviderType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
    #[serde(default = "default_expiry_days")]
    pub expiry_days: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthProviderType {
    Oauth2ClientCredentials,
    ApiKey,
    ServiceAccount,
}

impl AuthProviderType {
    pub fn as_str(self) -> &'static str {
        match self {
            AuthProviderType::Oauth2ClientCredentials => "oauth2_client_credentials",
            AuthProviderType::ApiKey => "api_key",
            AuthProviderType::ServiceAccount => "service_account",
        }
    }

    pub fn choices() -> [&'static str; 3] {
        ["oauth2_client_credentials", "api_key", "service_account"]
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => AuthProviderType::ApiKey,
            2 => AuthProviderType::ServiceAccount,
            _ => AuthProviderType::Oauth2ClientCredentials,
        }
    }
}

pub fn default_expiry_days() -> u32 {
    90
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_config_round_trips() {
        let config = AuthConfig {
            user_pool: Some(AuthUserPool::Shared),
            providers: vec![AuthProvider {
                name: "cognito-default".to_string(),
                type_: AuthProviderType::Oauth2ClientCredentials,
                audience: Some("https://api.tachyon.cloud".to_string()),
                expiry_days: 90,
                secret_ref: Some(".tachyon/credentials.json#cognito-default".to_string()),
            }],
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: AuthConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed, config);
        assert!(yaml.contains("user_pool: shared"));
        assert!(yaml.contains("type: oauth2_client_credentials"));
    }
}
