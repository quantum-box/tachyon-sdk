use super::*;

// --- Scaling subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum ScalingCommand {
    /// Show current scaling configuration
    Get {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update scaling configuration
    Update {
        /// App ID or name
        app_id: Option<String>,
        /// Minimum number of instances
        #[arg(long)]
        min_instances: Option<i32>,
        /// Maximum number of instances
        #[arg(long)]
        max_instances: Option<i32>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ScalingConfigResponse {
    #[serde(default)]
    pub(super) min_instances: Option<i32>,
    #[serde(default)]
    pub(super) max_instances: Option<i32>,
}

#[derive(Debug, Serialize)]
pub(super) struct UpdateScalingRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) min_instances: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) max_instances: Option<i32>,
}

pub(super) async fn run_scaling_get(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    // Scaling info is part of app details; fetch app and display scaling-relevant fields
    let app: serde_json::Value = api.get(&format!("/v1/compute/apps/{app_id}")).await?;
    if json {
        return print_json(&app);
    }
    println!("App ID: {app_id}");
    if let Some(scaling) = app.get("scaling") {
        println!(
            "Min instances: {}",
            scaling
                .get("min_instances")
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
        println!(
            "Max instances: {}",
            scaling
                .get("max_instances")
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
    } else {
        println!("No scaling configuration found.");
    }
    Ok(())
}

pub(super) async fn run_scaling_update(
    api: &ApiClient,
    app_id: &str,
    min_instances: Option<i32>,
    max_instances: Option<i32>,
) -> Result<()> {
    if min_instances.is_none() && max_instances.is_none() {
        return Err(anyhow!(
            "at least one of --min-instances or --max-instances is required"
        ));
    }
    let req = UpdateScalingRequest {
        min_instances,
        max_instances,
    };
    let resp: ScalingConfigResponse = api
        .patch(&format!("/v1/compute/apps/{app_id}/scaling"), &req)
        .await?;
    println!("Scaling updated.");
    if let Some(min) = resp.min_instances {
        println!("Min instances: {min}");
    }
    if let Some(max) = resp.max_instances {
        println!("Max instances: {max}");
    }
    Ok(())
}
