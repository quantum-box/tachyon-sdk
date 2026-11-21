//! Workload Identity Federation credential mode.
//!
//! When `TACHYON_API_KEY` is not set and `TACHYON_WORKLOAD_IDENTITY_APP_ID`
//! is, every command authenticates with a short-lived token exchanged from
//! the workload's OIDC token (GitHub Actions or a token file/variable)
//! instead of a stored login. The token is exchanged once per process,
//! reused until shortly before it expires, and exchanged again after a 401.
//!
//! Token values are never written to logs. Inside GitHub Actions each new
//! token is registered with `::add-mask::` so the runner redacts it if a
//! later step prints it.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, bail, Result};
use tachyon_sdk::workload_identity::{
    SubjectTokenSource, WorkloadIdentityConfig, WorkloadIdentityCredentials,
};

use crate::resolve;

pub const ENV_APP_ID: &str = "TACHYON_WORKLOAD_IDENTITY_APP_ID";
pub const ENV_ENVIRONMENT: &str = "TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT";
pub const ENV_ACTION: &str = "TACHYON_WORKLOAD_IDENTITY_ACTION";
pub const ENV_AUDIENCE: &str = "TACHYON_WORKLOAD_IDENTITY_AUDIENCE";
pub const ENV_SUBJECT_TOKEN_FILE: &str = "TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_FILE";
pub const ENV_SUBJECT_TOKEN_ENV: &str = "TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV";
pub const ENV_TTL: &str = "TACHYON_WORKLOAD_IDENTITY_TTL";

static CREDENTIALS: OnceLock<Arc<WorkloadIdentityCredentials>> = OnceLock::new();

/// Choose where the subject token comes from: an explicit file, then an
/// explicit variable, then the GitHub Actions OIDC service.
pub fn subject_token_source(
    file: Option<PathBuf>,
    env: Option<String>,
    audience: Option<&str>,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<SubjectTokenSource> {
    if let Some(path) = file {
        return Ok(SubjectTokenSource::File(path));
    }
    if let Some(name) = env {
        return Ok(SubjectTokenSource::Env(name));
    }
    SubjectTokenSource::github_actions_from_lookup(audience, lookup).ok_or_else(|| {
        anyhow!(
            "no subject token: pass --subject-token-file or --subject-token-env \
             (or set {ENV_SUBJECT_TOKEN_FILE} / {ENV_SUBJECT_TOKEN_ENV}), or run in \
             GitHub Actions with `permissions: id-token: write` so \
             ACTIONS_ID_TOKEN_REQUEST_URL and ACTIONS_ID_TOKEN_REQUEST_TOKEN are set"
        )
    })
}

/// Operator ID for the exchange. The exchange is unauthenticated, so a
/// tenant name or alias cannot be resolved first; it must already be an ID.
pub fn operator_id(tenant: &str) -> Result<String> {
    let tenant = tenant.trim();
    if tenant.is_empty() {
        bail!(
            "workload identity needs the tenant ID: pass --tenant-id <tn_...> or set \
             TACHYON_TENANT_ID"
        );
    }
    if !resolve::looks_like_id(tenant) {
        bail!(
            "workload identity needs the tenant ID (tn_...), not the name or alias \
             '{tenant}', because names cannot be resolved before authentication"
        );
    }
    Ok(tenant.to_string())
}

pub fn parse_ttl(value: &str) -> Result<i64> {
    value
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|ttl| *ttl > 0)
        .ok_or_else(|| anyhow!("{ENV_TTL} must be a positive number of seconds, got '{value}'"))
}

/// Build the credential-mode configuration from environment variables, or
/// `None` when `TACHYON_WORKLOAD_IDENTITY_APP_ID` is unset.
pub fn config_from_env(
    api_url: &str,
    tenant: &str,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<Option<WorkloadIdentityConfig>> {
    let get = |name: &str| lookup(name).filter(|value| !value.trim().is_empty());
    let Some(app_id) = get(ENV_APP_ID) else {
        return Ok(None);
    };
    let required = |name: &str| {
        get(name).ok_or_else(|| anyhow!("{ENV_APP_ID} is set, so {name} must be set too"))
    };
    let environment = required(ENV_ENVIRONMENT)?;
    let action = required(ENV_ACTION)?;
    let ttl_seconds = get(ENV_TTL).map(|value| parse_ttl(&value)).transpose()?;
    let audience = get(ENV_AUDIENCE);
    let subject_token = subject_token_source(
        get(ENV_SUBJECT_TOKEN_FILE).map(PathBuf::from),
        get(ENV_SUBJECT_TOKEN_ENV),
        audience.as_deref(),
        &lookup,
    )?;
    Ok(Some(WorkloadIdentityConfig {
        api_url: api_url.to_string(),
        app_id: app_id.trim().to_string(),
        operator_id: operator_id(tenant)?,
        environment: environment.trim().to_string(),
        action: action.trim().to_string(),
        subject_token,
        ttl_seconds,
    }))
}

/// Enable credential mode for this process. Returns whether it is active.
/// Call only when no API key was given: an API key always wins.
pub fn configure_from_env(api_url: &str, tenant: &str) -> Result<bool> {
    let Some(config) = config_from_env(api_url, tenant, |name| std::env::var(name).ok())? else {
        return Ok(false);
    };
    let credentials = WorkloadIdentityCredentials::new(reqwest12::Client::new(), config);
    let _ = CREDENTIALS.set(Arc::new(credentials));
    Ok(true)
}

pub fn is_active() -> bool {
    CREDENTIALS.get().is_some()
}

/// Register `token` with the GitHub Actions log masker. Workflow commands
/// are read from stderr as well as stdout, and stderr keeps the command out
/// of `$(...)` captures of stdout.
pub fn mask_in_github_actions(token: &str) {
    if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        eprintln!("::add-mask::{token}");
    }
}

/// Token for API calls in credential mode, or `None` when the mode is off.
pub async fn current_token() -> Option<Result<String>> {
    let credentials = CREDENTIALS.get()?;
    Some(
        credentials
            .access_token()
            .await
            .inspect(|token| mask_in_github_actions(token))
            .map_err(|err| anyhow!("workload identity token exchange failed: {err}")),
    )
}

/// Exchange a new token after the API answered 401. `None` when the mode
/// is off or the exchange failed (the failure is reported on stderr).
pub async fn refresh_after_401() -> Option<String> {
    let credentials = CREDENTIALS.get()?;
    match credentials.refresh().await {
        Ok(token) => {
            mask_in_github_actions(&token);
            eprintln!("Workload identity token re-exchanged after 401.");
            Some(token)
        }
        Err(err) => {
            eprintln!("Warning: workload identity token exchange after 401 failed: {err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lookup(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |name| vars.get(name).cloned()
    }

    const TENANT: &str = "tn_01hjryxysgey07h5jz5wagqj0m";

    #[test]
    fn disabled_without_app_id() {
        assert!(config_from_env("http://api", TENANT, lookup(&[]))
            .unwrap()
            .is_none());
    }

    #[test]
    fn requires_environment_and_action() {
        let err = config_from_env("http://api", TENANT, lookup(&[(ENV_APP_ID, "app_1")]))
            .unwrap_err()
            .to_string();
        assert!(err.contains(ENV_ENVIRONMENT));
    }

    #[test]
    fn uses_github_actions_when_no_explicit_source() {
        let config = config_from_env(
            "http://api",
            TENANT,
            lookup(&[
                (ENV_APP_ID, "app_1"),
                (ENV_ENVIRONMENT, "production"),
                (ENV_ACTION, "compute:CreateDeployment"),
                (ENV_TTL, "600"),
                ("ACTIONS_ID_TOKEN_REQUEST_URL", "https://gh.test/t?x=1"),
                ("ACTIONS_ID_TOKEN_REQUEST_TOKEN", "req"),
            ]),
        )
        .unwrap()
        .unwrap();
        assert_eq!(config.operator_id, TENANT);
        assert_eq!(config.ttl_seconds, Some(600));
        assert!(matches!(
            config.subject_token,
            SubjectTokenSource::GithubActions { ref audience, .. }
                if audience == tachyon_sdk::workload_identity::DEFAULT_AUDIENCE
        ));
    }

    #[test]
    fn explicit_file_wins_over_github_actions() {
        let source = subject_token_source(
            Some(PathBuf::from("/var/run/token")),
            Some("OIDC".into()),
            None,
            lookup(&[
                ("ACTIONS_ID_TOKEN_REQUEST_URL", "https://gh.test/t"),
                ("ACTIONS_ID_TOKEN_REQUEST_TOKEN", "req"),
            ]),
        )
        .unwrap();
        assert!(matches!(source, SubjectTokenSource::File(_)));
    }

    #[test]
    fn missing_source_explains_id_token_permission() {
        let err = subject_token_source(None, None, None, lookup(&[]))
            .unwrap_err()
            .to_string();
        assert!(err.contains("id-token: write"));
    }

    #[test]
    fn operator_must_be_an_id() {
        assert!(operator_id("").is_err());
        assert!(operator_id("my-tenant").is_err());
        assert_eq!(operator_id(TENANT).unwrap(), TENANT);
    }

    #[test]
    fn ttl_must_be_positive() {
        assert_eq!(parse_ttl("300").unwrap(), 300);
        assert!(parse_ttl("0").is_err());
        assert!(parse_ttl("abc").is_err());
    }
}
