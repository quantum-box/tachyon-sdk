use std::{collections::HashMap, fmt::Debug, sync::RwLock};

use crate::auth::AuthResult;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecretPath(String);

impl SecretPath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SecretPath {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[async_trait::async_trait]
pub trait SecretsApp: Debug + Send + Sync + 'static {
    async fn get_secret(
        &self,
        path: &SecretPath,
    ) -> AuthResult<Option<String>>;

    async fn put_secret(
        &self,
        path: &SecretPath,
        value: &str,
    ) -> AuthResult<()>;

    async fn delete_secret(&self, path: &SecretPath) -> AuthResult<()>;

    async fn list_secrets(
        &self,
        prefix: Option<&str>,
    ) -> AuthResult<Vec<SecretPath>>;
}

#[derive(Debug, Default)]
pub struct LocalSecretsApp {
    secrets: RwLock<HashMap<String, String>>,
}

#[async_trait::async_trait]
impl SecretsApp for LocalSecretsApp {
    async fn get_secret(
        &self,
        path: &SecretPath,
    ) -> AuthResult<Option<String>> {
        Ok(self.secrets.read().unwrap().get(path.as_str()).cloned())
    }

    async fn put_secret(
        &self,
        path: &SecretPath,
        value: &str,
    ) -> AuthResult<()> {
        self.secrets
            .write()
            .unwrap()
            .insert(path.as_str().to_string(), value.to_string());
        Ok(())
    }

    async fn delete_secret(&self, path: &SecretPath) -> AuthResult<()> {
        self.secrets.write().unwrap().remove(path.as_str());
        Ok(())
    }

    async fn list_secrets(
        &self,
        prefix: Option<&str>,
    ) -> AuthResult<Vec<SecretPath>> {
        Ok(self
            .secrets
            .read()
            .unwrap()
            .keys()
            .filter(|path| {
                prefix
                    .map(|prefix| path.starts_with(prefix))
                    .unwrap_or(true)
            })
            .cloned()
            .map(SecretPath::from)
            .collect())
    }
}

pub async fn build_secrets_app_from_env() -> std::sync::Arc<dyn SecretsApp>
{
    std::sync::Arc::new(LocalSecretsApp::default())
}
