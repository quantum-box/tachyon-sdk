/// Runtime configuration loaded from env vars and CLI flags.
#[derive(Debug, Clone)]
pub struct Config {
    pub api_url: String,
    pub tenant_id: String,
    pub auth_token: Option<String>,
}
