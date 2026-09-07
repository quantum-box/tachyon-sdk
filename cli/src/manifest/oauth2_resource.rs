//! `kind: OAuth2Resource` — an OAuth2 protected resource (RFC 8707 resource
//! indicator target) managed through `tachyon manifest` (PLT-4373).
//!
//! The typed model mirrors the server-side spec in
//! `packages/iac/src/domain/oauth2_resource_manifest.rs` (quantum-box/
//! tachyon-apps). Plan and apply go through `/v1/auth/oauth2-resources`,
//! which is the same registry `tachyon iac apply` writes to, so the two
//! paths converge on one row per `(tenant, metadata.name)`.

use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::ApiClient;

/// An `OAuth2Resource` manifest document.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(title = "OAuth2ResourceManifest")]
pub struct OAuth2ResourceManifest {
    /// Manifest API version. Always `apps.tachy.one/v1alpha`.
    pub api_version: OAuth2ResourceApiVersion,
    pub kind: OAuth2ResourceKind,
    pub metadata: OAuth2ResourceMetadata,
    pub spec: OAuth2ResourceSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum OAuth2ResourceApiVersion {
    #[serde(rename = "apps.tachy.one/v1alpha")]
    V1Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum OAuth2ResourceKind {
    OAuth2Resource,
}

/// Manifest metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2ResourceMetadata {
    /// Registry name, unique per tenant. Re-applying the same name updates
    /// the existing registration.
    pub name: String,
    /// Tenant that owns the resource registration (e.g. `tn_01hj...`).
    /// When set it must match the CLI's active tenant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// Cross-document ordering hints, e.g. `["OAuth2Client/my-client"]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
}

/// Declarative spec of the protected resource.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2ResourceSpec {
    /// Canonical resource URI: absolute `http(s)` URI without a fragment.
    /// Must equal the `resource` in the resource server's Protected
    /// Resource Metadata (RFC 9728). Unique across all tenants.
    pub resource: String,
    /// Human-readable name shown on the consent page (defaults to the URI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Scopes an access token bound to this resource may carry. A request
    /// asking for more is rejected with `invalid_target`.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// Which OAuth2 clients may request tokens for this resource.
    #[serde(default)]
    pub clients: OAuth2ResourceClientsSpec,
}

/// Client admission rules.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2ResourceClientsSpec {
    /// Allow clients registered through RFC 7591 dynamic registration
    /// (MCP clients). Default `false`.
    #[serde(default)]
    pub allow_dynamic_registration: bool,
    /// `OAuth2Client` manifest names in the same tenant.
    #[serde(default)]
    pub names: Vec<String>,
}

pub(crate) fn parse(document: &Value) -> Result<OAuth2ResourceManifest> {
    let manifest: OAuth2ResourceManifest = serde_json::from_value(document.clone())
        .map_err(|error| anyhow!("invalid OAuth2Resource manifest: {error}"))?;
    validate(&manifest)?;
    Ok(manifest)
}

/// Local validation mirroring the server rules so `manifest validate`
/// rejects the same documents without an API call.
pub(crate) fn validate(manifest: &OAuth2ResourceManifest) -> Result<()> {
    if manifest.metadata.name.trim().is_empty() {
        return Err(anyhow!("OAuth2Resource metadata.name must not be empty"));
    }
    validate_resource_uri(&manifest.spec.resource)?;
    if manifest
        .spec
        .display_name
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(anyhow!(
            "OAuth2Resource spec.displayName must not be blank when declared"
        ));
    }
    validate_unique_tokens(&manifest.spec.scopes, "spec.scopes")?;
    validate_unique_tokens(&manifest.spec.clients.names, "spec.clients.names")?;
    Ok(())
}

fn validate_resource_uri(uri: &str) -> Result<()> {
    let reject = |reason: &str| Err(anyhow!("OAuth2Resource spec.resource '{uri}': {reason}"));
    if uri.is_empty() {
        return reject("must not be empty");
    }
    if uri.len() > 512 {
        return reject("exceeds 512 characters");
    }
    if uri.chars().any(char::is_whitespace) {
        return reject("must not contain whitespace");
    }
    if uri.contains('#') {
        return reject("must not contain a fragment");
    }
    let Some(rest) = uri
        .strip_prefix("https://")
        .or_else(|| uri.strip_prefix("http://"))
    else {
        return reject("must be an absolute http(s) URI");
    };
    if rest.split(['/', '?']).next().unwrap_or("").is_empty() {
        return reject("must include a host");
    }
    Ok(())
}

fn validate_unique_tokens(values: &[String], field: &str) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for value in values {
        if value.is_empty() || value.chars().any(char::is_whitespace) {
            return Err(anyhow!(
                "OAuth2Resource {field} entries must be non-empty and contain no whitespace: '{value}'"
            ));
        }
        if !seen.insert(value.as_str()) {
            return Err(anyhow!("OAuth2Resource {field} contains duplicate '{value}'"));
        }
    }
    Ok(())
}

// ── REST models (`/v1/auth/oauth2-resources`) ─────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResourceResponse {
    pub id: String,
    pub name: String,
    pub resource: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub allow_dynamic_clients: bool,
    #[serde(default)]
    pub allowed_client_names: Vec<String>,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceListResponse {
    resources: Vec<ResourceResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    resource: &'a str,
    display_name: Option<&'a str>,
    scopes: &'a [String],
    allow_dynamic_clients: bool,
    allowed_client_names: &'a [String],
}

impl<'a> ResourceBody<'a> {
    fn from_spec(name: Option<&'a str>, spec: &'a OAuth2ResourceSpec) -> Self {
        Self {
            name,
            resource: &spec.resource,
            display_name: spec.display_name.as_deref(),
            scopes: &spec.scopes,
            allow_dynamic_clients: spec.clients.allow_dynamic_registration,
            allowed_client_names: &spec.clients.names,
        }
    }
}

// ── Plan / apply ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChangeKind {
    Create,
    Update,
    Unchanged,
}

#[derive(Debug, Serialize)]
pub(crate) struct OAuth2ResourcePlan {
    pub name: String,
    pub resource: String,
    pub change: ChangeKind,
    /// Field-level differences (`field: live -> desired`), empty when
    /// unchanged or created.
    pub diff: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct OAuth2ResourceApplyResult {
    pub name: String,
    pub resource: String,
    pub id: String,
    pub outcome: ChangeKind,
}

/// Compare the desired spec with the live registration.
pub(crate) fn classify(live: Option<&ResourceResponse>, spec: &OAuth2ResourceSpec) -> (ChangeKind, Vec<String>) {
    let Some(live) = live else {
        return (ChangeKind::Create, Vec::new());
    };
    let mut diff = Vec::new();
    if live.resource != spec.resource {
        diff.push(format!("resource: {} -> {}", live.resource, spec.resource));
    }
    if live.display_name != spec.display_name {
        diff.push(format!(
            "displayName: {} -> {}",
            live.display_name.as_deref().unwrap_or("<none>"),
            spec.display_name.as_deref().unwrap_or("<none>")
        ));
    }
    if live.scopes != spec.scopes {
        diff.push(format!("scopes: {:?} -> {:?}", live.scopes, spec.scopes));
    }
    if live.allow_dynamic_clients != spec.clients.allow_dynamic_registration {
        diff.push(format!(
            "clients.allowDynamicRegistration: {} -> {}",
            live.allow_dynamic_clients, spec.clients.allow_dynamic_registration
        ));
    }
    if live.allowed_client_names != spec.clients.names {
        diff.push(format!(
            "clients.names: {:?} -> {:?}",
            live.allowed_client_names, spec.clients.names
        ));
    }
    if diff.is_empty() {
        (ChangeKind::Unchanged, diff)
    } else {
        (ChangeKind::Update, diff)
    }
}

fn check_tenant(manifest: &OAuth2ResourceManifest, tenant_id: &str) -> Result<()> {
    match manifest.metadata.tenant_id.as_deref() {
        Some(declared) if declared != tenant_id => Err(anyhow!(
            "OAuth2Resource '{}' declares tenantId {declared} but the active tenant is {tenant_id}; \
             switch tenant or fix metadata.tenantId",
            manifest.metadata.name
        )),
        _ => Ok(()),
    }
}

async fn find_live(api: &ApiClient, name: &str) -> Result<Option<ResourceResponse>> {
    let live: ResourceListResponse = api.get("/v1/auth/oauth2-resources").await?;
    Ok(live.resources.into_iter().find(|resource| resource.name == name))
}

pub(crate) async fn plan(
    api: &ApiClient,
    manifest: &OAuth2ResourceManifest,
    tenant_id: &str,
) -> Result<OAuth2ResourcePlan> {
    check_tenant(manifest, tenant_id)?;
    let live = find_live(api, &manifest.metadata.name).await?;
    let (change, diff) = classify(live.as_ref(), &manifest.spec);
    Ok(OAuth2ResourcePlan {
        name: manifest.metadata.name.clone(),
        resource: manifest.spec.resource.clone(),
        change,
        diff,
    })
}

pub(crate) async fn apply(
    api: &ApiClient,
    manifest: &OAuth2ResourceManifest,
    tenant_id: &str,
) -> Result<OAuth2ResourceApplyResult> {
    check_tenant(manifest, tenant_id)?;
    let name = manifest.metadata.name.as_str();
    let live = find_live(api, name).await?;
    let (change, _) = classify(live.as_ref(), &manifest.spec);
    let stored: ResourceResponse = match (change, live) {
        (ChangeKind::Unchanged, Some(live)) => live,
        (ChangeKind::Update, Some(live)) => {
            api.put(
                &format!("/v1/auth/oauth2-resources/{}", live.id),
                &ResourceBody::from_spec(None, &manifest.spec),
            )
            .await?
        }
        _ => {
            api.post(
                "/v1/auth/oauth2-resources",
                &ResourceBody::from_spec(Some(name), &manifest.spec),
            )
            .await?
        }
    };
    Ok(OAuth2ResourceApplyResult {
        name: name.to_string(),
        resource: stored.resource,
        id: stored.id,
        outcome: change,
    })
}

pub(crate) fn print_plan(plan: &OAuth2ResourcePlan) {
    let symbol = match plan.change {
        ChangeKind::Create => "+",
        ChangeKind::Update => "~",
        ChangeKind::Unchanged => "=",
    };
    println!(
        "  {symbol} OAuth2Resource/{} ({}) {:?}",
        plan.name, plan.resource, plan.change
    );
    for line in &plan.diff {
        println!("      {line}");
    }
}

pub(crate) fn print_apply(result: &OAuth2ResourceApplyResult) {
    println!(
        "  OAuth2Resource/{} ({}) {:?} [{}]",
        result.name, result.resource, result.outcome, result.id
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(spec: Value) -> Value {
        serde_json::json!({
            "apiVersion": "apps.tachy.one/v1alpha",
            "kind": "OAuth2Resource",
            "metadata": { "name": "library-mcp", "tenantId": "tn_01hjryxysgey07h5jz5wagqj0m" },
            "spec": spec
        })
    }

    fn live(display_name: Option<&str>) -> ResourceResponse {
        ResourceResponse {
            id: "ors_01live".into(),
            name: "library-mcp".into(),
            resource: "https://library.example/mcp".into(),
            display_name: display_name.map(str::to_string),
            scopes: vec!["mcp:read".into(), "mcp:write".into()],
            allow_dynamic_clients: true,
            allowed_client_names: vec![],
            status: "active".into(),
        }
    }

    #[test]
    fn parses_and_validates_full_manifest() {
        let parsed = parse(&manifest(serde_json::json!({
            "resource": "https://library.example/mcp",
            "displayName": "Library MCP",
            "scopes": ["mcp:read", "mcp:write"],
            "clients": { "allowDynamicRegistration": true, "names": ["console"] }
        })))
        .unwrap();
        assert_eq!(parsed.spec.clients.names, vec!["console".to_string()]);
        assert!(parsed.spec.clients.allow_dynamic_registration);
    }

    #[test]
    fn rejects_invalid_uri_and_duplicates() {
        for bad in ["library.example/mcp", "https://library.example/mcp#frag", "https:///mcp"] {
            assert!(
                parse(&manifest(serde_json::json!({ "resource": bad }))).is_err(),
                "{bad}"
            );
        }
        assert!(parse(&manifest(serde_json::json!({
            "resource": "https://library.example/mcp",
            "scopes": ["mcp:read", "mcp:read"]
        })))
        .is_err());
    }

    #[test]
    fn classify_reports_create_update_unchanged() {
        let spec: OAuth2ResourceSpec = serde_json::from_value(serde_json::json!({
            "resource": "https://library.example/mcp",
            "scopes": ["mcp:read", "mcp:write"],
            "clients": { "allowDynamicRegistration": true }
        }))
        .unwrap();
        assert_eq!(classify(None, &spec).0, ChangeKind::Create);
        assert_eq!(classify(Some(&live(None)), &spec).0, ChangeKind::Unchanged);
        let (change, diff) = classify(Some(&live(Some("Library MCP"))), &spec);
        assert_eq!(change, ChangeKind::Update);
        assert_eq!(diff, vec!["displayName: Library MCP -> <none>".to_string()]);
    }

    #[test]
    fn tenant_mismatch_is_rejected() {
        let parsed = parse(&manifest(serde_json::json!({ "resource": "https://library.example/mcp" })))
            .unwrap();
        assert!(check_tenant(&parsed, "tn_01hjjn348rn3t49zz6hvmfq67p").is_err());
        assert!(check_tenant(&parsed, "tn_01hjryxysgey07h5jz5wagqj0m").is_ok());
    }

    #[test]
    fn schema_generates() {
        let schema = schemars::schema_for!(OAuth2ResourceManifest);
        let rendered = serde_json::to_string(&schema).unwrap();
        assert!(rendered.contains("allowDynamicRegistration"));
        assert!(rendered.contains("OAuth2Resource"));
    }
}
