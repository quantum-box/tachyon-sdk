use super::*;

// --- Deployments subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum DeploymentsCommand {
    /// List deployments for an app
    List {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get details of a specific deployment
    Get {
        /// Deployment ID
        deployment_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Rollback an app to a previous deployment
    Rollback {
        /// App ID or name
        app_id: Option<String>,
        /// Deployment ID to roll back to
        #[arg(long)]
        deployment_id: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ListDeploymentsResponse {
    pub(super) deployments: Vec<DeploymentResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct DeploymentResponse {
    pub(super) id: String,
    #[serde(default)]
    pub(super) app_id: Option<String>,
    #[serde(default)]
    pub(super) build_id: Option<String>,
    #[serde(default)]
    pub(super) pr_number: Option<i32>,
    pub(super) status: String,
    #[serde(default)]
    pub(super) source_branch: Option<String>,
    #[serde(default)]
    pub(super) public_url: Option<String>,
    #[serde(default)]
    pub(super) url: Option<String>,
    #[serde(default)]
    pub(super) created_at: Option<String>,
    #[serde(default)]
    pub(super) updated_at: Option<String>,
}

impl DeploymentResponse {
    pub(super) fn display_url(&self) -> String {
        self.display_url_with_pr(self.pr_number)
    }

    pub(super) fn display_url_with_pr(&self, pr_number: Option<i32>) -> String {
        if let Some(public_url) = self.public_url.as_deref() {
            return public_url.to_string();
        }

        let Some(url) = self.url.as_deref() else {
            return "-".to_string();
        };

        public_preview_url_from_pages_url(url, pr_number).unwrap_or_else(|| url.to_string())
    }
}

fn public_preview_url_from_pages_url(url: &str, pr_number: Option<i32>) -> Option<String> {
    let pr_number = pr_number?;
    let host = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url);
    let pages_project = host.strip_suffix(".pages.dev")?;
    let app_name = pages_project.rsplit_once('.')?.1;

    Some(format!("https://pr{pr_number}--{app_name}.txcloud.app"))
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct RollbackRequest {
    pub(super) deployment_id: String,
}

pub(super) async fn run_deployments_list(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    let resp: ListDeploymentsResponse = api
        .get(&format!("/v1/compute/apps/{app_id}/deployments"))
        .await?;
    if json {
        return print_json(&resp.deployments);
    }
    if resp.deployments.is_empty() {
        println!("No deployments found for app {app_id}");
        return Ok(());
    }
    println!(
        "{:<28}  {:<12}  {:<28}  {:<40}  CREATED AT",
        "DEPLOYMENT ID", "STATUS", "BUILD ID", "URL"
    );
    println!(
        "{:-<28}  {:-<12}  {:-<28}  {:-<40}  {:-<19}",
        "", "", "", "", ""
    );
    for dep in &resp.deployments {
        println!(
            "{:<28}  {:<12}  {:<28}  {:<40}  {}",
            dep.id,
            dep.status,
            dep.build_id.as_deref().unwrap_or("-"),
            truncate(&dep.display_url(), 40),
            dep.created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    Ok(())
}

pub(super) async fn run_deployments_get(
    api: &ApiClient,
    deployment_id: &str,
    json: bool,
) -> Result<()> {
    let dep: DeploymentResponse = api
        .get(&format!("/v1/compute/deployments/{deployment_id}"))
        .await?;
    if json {
        return print_json(&dep);
    }
    println!("ID:       {}", dep.id);
    println!("Status:   {}", dep.status);
    println!("App ID:   {}", dep.app_id.as_deref().unwrap_or("-"));
    println!("Build ID: {}", dep.build_id.as_deref().unwrap_or("-"));
    println!("URL:      {}", dep.display_url());
    println!(
        "Created:  {}",
        dep.created_at
            .as_deref()
            .map(format_created_at)
            .unwrap_or_else(|| "-".to_string())
    );
    Ok(())
}

pub(super) async fn run_deployments_rollback(
    api: &ApiClient,
    app_id: &str,
    deployment_id: &str,
) -> Result<()> {
    let req = RollbackRequest {
        deployment_id: deployment_id.to_string(),
    };
    let dep: DeploymentResponse = api
        .post(&format!("/v1/compute/apps/{app_id}/rollback"), &req)
        .await?;
    println!("Rollback initiated. New deployment: {}", dep.id);
    println!("Status: {}", dep.status);
    Ok(())
}
