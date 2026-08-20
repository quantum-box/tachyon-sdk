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
        #[arg(long, conflicts_with = "to_previous")]
        deployment_id: Option<String>,
        /// Roll back to the newest production rollback candidate
        #[arg(long)]
        to_previous: bool,
    },
    /// List production rollback candidates (newest first)
    RollbackCandidates {
        /// App ID or name
        app_id: Option<String>,
        /// Maximum number of candidates to return
        #[arg(long)]
        limit: Option<usize>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
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
    /// Why a failed deployment failed. The API has always returned this;
    /// without it a `failed` deployment is opaque from the CLI and the
    /// only way to see the cause is to query the API by hand.
    #[serde(default)]
    pub(super) error_message: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) deployment_id: Option<String>,
    #[serde(default)]
    pub(super) to_previous: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ListRollbackCandidatesResponse {
    pub(super) candidates: Vec<DeploymentResponse>,
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
    if let Some(error) = dep.error_message.as_deref().filter(|e| !e.is_empty()) {
        println!("Error:    {error}");
    }
    Ok(())
}

pub(super) async fn run_deployments_rollback(
    api: &ApiClient,
    app_id: &str,
    deployment_id: Option<&str>,
    to_previous: bool,
) -> Result<()> {
    if deployment_id.is_none() && !to_previous {
        anyhow::bail!(
            "specify a rollback target: --deployment-id <id> or --to-previous\n\
             hint: `tachyon compute deployments rollback-candidates` lists the eligible targets"
        );
    }
    let req = RollbackRequest {
        deployment_id: deployment_id.map(str::to_string),
        to_previous,
    };
    let dep: DeploymentResponse = api
        .post(&format!("/v1/compute/apps/{app_id}/rollback"), &req)
        .await?;
    if to_previous {
        println!(
            "Rolled back to previous production version (build {}).",
            dep.build_id.as_deref().unwrap_or("-")
        );
    }
    println!("Rollback initiated. New deployment: {}", dep.id);
    println!("Status: {}", dep.status);
    Ok(())
}

pub(super) async fn run_deployments_rollback_candidates(
    api: &ApiClient,
    app_id: &str,
    limit: Option<usize>,
    json: bool,
) -> Result<()> {
    let mut path = format!("/v1/compute/apps/{app_id}/rollback/candidates");
    if let Some(limit) = limit {
        path.push_str(&format!("?limit={limit}"));
    }
    let resp: ListRollbackCandidatesResponse = api.get(&path).await?;
    if json {
        return print_json(&resp.candidates);
    }
    if resp.candidates.is_empty() {
        println!(
            "No rollback candidates for app {app_id}: no previous production \
             deployment has both a successful serving record and the provider \
             metadata required for a build-less rollback."
        );
        return Ok(());
    }
    println!(
        "{:<28}  {:<12}  {:<20}  {:<28}  CREATED AT",
        "DEPLOYMENT ID", "STATUS", "BRANCH", "BUILD ID"
    );
    println!(
        "{:-<28}  {:-<12}  {:-<20}  {:-<28}  {:-<19}",
        "", "", "", "", ""
    );
    for dep in &resp.candidates {
        println!(
            "{:<28}  {:<12}  {:<20}  {:<28}  {}",
            dep.id,
            dep.status,
            truncate(dep.source_branch.as_deref().unwrap_or("-"), 20),
            dep.build_id.as_deref().unwrap_or("-"),
            dep.created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    println!();
    println!(
        "`rollback --to-previous` rolls back to the first candidate ({}).",
        resp.candidates[0].id
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deployment_response_keeps_the_failure_reason() {
        // A failed deployment is opaque without this: the API returns the
        // reason, and dropping it here sent people to the raw API to find
        // out why a deploy failed.
        let dep: DeploymentResponse = serde_json::from_str(
            r#"{
                "id": "dep_1",
                "status": "failed",
                "error_message": "NotFoundError: Lambda function lambda-x does not exist"
            }"#,
        )
        .unwrap();

        assert_eq!(
            dep.error_message.as_deref(),
            Some("NotFoundError: Lambda function lambda-x does not exist")
        );
    }

    #[test]
    fn deployment_response_tolerates_a_missing_failure_reason() {
        let dep: DeploymentResponse =
            serde_json::from_str(r#"{"id": "dep_1", "status": "active"}"#).unwrap();

        assert!(dep.error_message.is_none());
    }
}
