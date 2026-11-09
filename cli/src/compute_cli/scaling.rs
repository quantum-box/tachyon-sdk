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
        /// Lambda memory size in MB (128-10240)
        #[arg(long)]
        lambda_memory_size: Option<i32>,
        /// Lambda invocation timeout in seconds (1-900).
        ///
        /// Raise this when a deploy hook invokes the app itself and needs
        /// longer than the 30s default -- a migration gate over a large
        /// schema is the usual reason.
        #[arg(long)]
        lambda_timeout: Option<i32>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ScalingConfigResponse {
    #[serde(default)]
    pub(super) min_instances: Option<i32>,
    #[serde(default)]
    pub(super) max_instances: Option<i32>,
    #[serde(default)]
    pub(super) lambda_memory_size: Option<i32>,
    #[serde(default)]
    pub(super) lambda_timeout: Option<i32>,
}

/// Every field is omitted when unset. The endpoint merges what it receives,
/// so sending an untouched setting as null would clear it.
#[derive(Debug, Serialize)]
pub(super) struct UpdateScalingRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) min_instances: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) max_instances: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) lambda_memory_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) lambda_timeout: Option<i32>,
}

pub(super) async fn run_scaling_get(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    // Scaling info is part of app details; fetch app and display scaling-relevant fields
    let app: serde_json::Value = api.get(&format!("/v1/compute/apps/{app_id}")).await?;
    if json {
        return print_json(&app);
    }
    println!("App ID: {app_id}");
    if let Some(scaling) = app.get("scaling") {
        for (label, field) in [
            ("Min instances", "min_instances"),
            ("Max instances", "max_instances"),
            ("Lambda memory size (MB)", "lambda_memory_size"),
            ("Lambda timeout (s)", "lambda_timeout"),
        ] {
            println!(
                "{label}: {}",
                scaling
                    .get(field)
                    .and_then(|value| value.as_i64())
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string())
            );
        }
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
    lambda_memory_size: Option<i32>,
    lambda_timeout: Option<i32>,
) -> Result<()> {
    if min_instances.is_none()
        && max_instances.is_none()
        && lambda_memory_size.is_none()
        && lambda_timeout.is_none()
    {
        return Err(anyhow!(
            "at least one of --min-instances, --max-instances, \
             --lambda-memory-size, or --lambda-timeout is required"
        ));
    }
    // Reject out-of-range values here so the mistake is named locally
    // instead of coming back as a generic 400.
    if let Some(timeout) = lambda_timeout {
        if !(1..=900).contains(&timeout) {
            return Err(anyhow!(
                "--lambda-timeout must be between 1 and 900 seconds, got {timeout}"
            ));
        }
    }
    if let Some(memory_size) = lambda_memory_size {
        if !(128..=10240).contains(&memory_size) {
            return Err(anyhow!(
                "--lambda-memory-size must be between 128 and 10240 MB, got {memory_size}"
            ));
        }
    }
    let req = UpdateScalingRequest {
        min_instances,
        max_instances,
        lambda_memory_size,
        lambda_timeout,
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
    if let Some(memory_size) = resp.lambda_memory_size {
        println!("Lambda memory size (MB): {memory_size}");
    }
    if let Some(timeout) = resp.lambda_timeout {
        println!("Lambda timeout (s): {timeout}");
    }
    Ok(())
}
