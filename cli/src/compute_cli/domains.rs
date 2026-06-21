use super::*;

// --- Domains subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum DomainsCommand {
    /// List custom domains for an app
    List {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a custom domain
    Add {
        /// App ID or name
        app_id: Option<String>,
        /// Domain name
        domain: String,
    },
    /// Verify a custom domain
    Verify {
        /// Domain ID
        domain_id: String,
    },
    /// Remove a custom domain
    Remove {
        /// Domain ID
        domain_id: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ListDomainsResponse {
    pub(super) domains: Vec<CustomDomainResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct CustomDomainResponse {
    pub(super) id: String,
    pub(super) domain: String,
    #[serde(default)]
    pub(super) status: Option<String>,
    #[serde(default)]
    pub(super) verified: Option<bool>,
    #[serde(default)]
    pub(super) created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct AddDomainRequest {
    pub(super) domain: String,
}

pub(super) async fn run_domains_list(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    let resp: ListDomainsResponse = api
        .get(&format!("/v1/compute/apps/{app_id}/domains"))
        .await?;
    if json {
        return print_json(&resp.domains);
    }
    if resp.domains.is_empty() {
        println!("No custom domains for app {app_id}");
        return Ok(());
    }
    println!(
        "{:<28}  {:<40}  {:<10}  {:<8}  CREATED AT",
        "ID", "DOMAIN", "STATUS", "VERIFIED"
    );
    println!(
        "{:-<28}  {:-<40}  {:-<10}  {:-<8}  {:-<19}",
        "", "", "", "", ""
    );
    for d in &resp.domains {
        println!(
            "{:<28}  {:<40}  {:<10}  {:<8}  {}",
            d.id,
            d.domain,
            d.status.as_deref().unwrap_or("-"),
            d.verified
                .map(|v| if v { "yes" } else { "no" })
                .unwrap_or("-"),
            d.created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    Ok(())
}

pub(super) async fn run_domains_add(api: &ApiClient, app_id: &str, domain: &str) -> Result<()> {
    let req = AddDomainRequest {
        domain: domain.to_string(),
    };
    let resp: CustomDomainResponse = api
        .post(&format!("/v1/compute/apps/{app_id}/domains"), &req)
        .await?;
    println!("Domain added: {} (ID: {})", resp.domain, resp.id);
    Ok(())
}

pub(super) async fn run_domains_verify(api: &ApiClient, domain_id: &str) -> Result<()> {
    api.post_no_body(&format!("/v1/compute/domains/{domain_id}/verify"))
        .await?;
    println!("Domain {domain_id} verification initiated.");
    Ok(())
}

pub(super) async fn run_domains_remove(api: &ApiClient, domain_id: &str) -> Result<()> {
    api.delete(&format!("/v1/compute/domains/{domain_id}"))
        .await?;
    println!("Domain {domain_id} removed.");
    Ok(())
}
