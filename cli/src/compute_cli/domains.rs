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
    pub(super) app_id: String,
    pub(super) domain: String,
    pub(super) status: String,
    pub(super) tls_status: String,
    pub(super) cname_target: String,
    pub(super) created_at: String,
    pub(super) updated_at: String,
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
        "{:<28}  {:<40}  {:<10}  {:<12}  CREATED AT",
        "ID", "DOMAIN", "STATUS", "TLS STATUS"
    );
    println!(
        "{:-<28}  {:-<40}  {:-<10}  {:-<12}  {:-<19}",
        "", "", "", "", ""
    );
    for d in &resp.domains {
        println!(
            "{:<28}  {:<40}  {:<10}  {:<12}  {}",
            d.id,
            d.domain,
            d.status,
            d.tls_status,
            format_created_at(&d.created_at),
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
