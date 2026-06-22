use super::*;

// --- Builds subcommands ---

#[derive(Debug, Clone, Args)]
pub struct PreviewArgs {
    #[command(subcommand)]
    pub command: Option<PreviewCommand>,
    /// App ID or name (legacy shortcut; use `preview create --app`)
    #[arg(hide = true)]
    pub app_id: Option<String>,
    /// Branch to build (legacy shortcut; use `preview create --branch`)
    #[arg(long, hide = true)]
    pub branch: Option<String>,
    /// Pull request number (legacy shortcut)
    #[arg(long, hide = true)]
    pub pr: Option<i32>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum PreviewCommand {
    /// Create a preview build for a branch
    Create {
        /// App ID or name
        #[arg(long)]
        app: String,
        /// Branch to build
        #[arg(long)]
        branch: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum BuildsCommand {
    /// List builds for an app
    List {
        /// App ID or name
        app_id: Option<String>,
        /// Maximum number of builds to display
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get details of a specific build
    Get {
        /// Build ID
        build_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Trigger a new build
    Trigger {
        /// App ID or name
        app_id: Option<String>,
        /// Branch to build (optional)
        #[arg(long)]
        branch: Option<String>,
        /// Pull request number to (re-)fire a preview build for.
        /// The PR's head branch and commit are resolved server-side.
        #[arg(long)]
        pr: Option<i32>,
    },
    /// Cancel a running build
    Cancel {
        /// Build ID
        build_id: String,
    },
    /// Fetch build logs
    Logs {
        /// App ID (used to resolve latest build if --build-id is not given)
        app_id: Option<String>,
        /// Build ID (defaults to the latest build)
        #[arg(long)]
        build_id: Option<String>,
        /// Keep polling until the build is complete
        #[arg(long)]
        follow: bool,
        /// Emit compact JSON Lines for coding agents
        #[arg(long)]
        agent: bool,
    },
    /// Watch build logs and final status until completion
    Watch {
        /// App ID or name (optional when --build-id is specified)
        app_id: Option<String>,
        /// Build ID (defaults to the latest build for the given app)
        #[arg(long)]
        build_id: Option<String>,
        /// Poll interval in seconds
        #[arg(long, default_value_t = 5)]
        interval_secs: u64,
        /// Maximum wait time in seconds
        #[arg(long)]
        timeout_secs: Option<u64>,
        /// Do not print build logs, only status/result
        #[arg(long)]
        no_logs: bool,
        /// Emit compact JSON Lines for coding agents
        #[arg(long)]
        agent: bool,
    },
    /// Reproduce a cloud build locally in Docker.
    ///
    /// Phase 1: requires `--mock <path>` pointing at a local build-config
    /// fixture (PLT-914). Phase 2 (PLT-913) will fetch the buildspec and
    /// environment from tachyon-api given the build id.
    Reproduce {
        /// Build ID to reproduce (informational in --mock mode).
        build_id: String,
        /// Path to a local mock build-config (json or yaml). Required until
        /// PLT-913 endpoint is available.
        #[arg(long)]
        mock: Option<PathBuf>,
        /// Source tree to mount into the container (defaults to cwd).
        #[arg(long)]
        source_dir: Option<PathBuf>,
        /// Override the CodeBuild image (e.g.
        /// public.ecr.aws/codebuild/amazonlinux-x86_64-standard:5.0).
        #[arg(long)]
        image: Option<String>,
        /// Print the docker invocation instead of running it.
        #[arg(long)]
        dry_run: bool,
    },
    /// Run a Cloud App build workload from a JobRun spec.
    #[command(hide = true)]
    RunJob {
        /// Environment variable that contains the BuildWorkloadSpec JSON.
        #[arg(long, default_value = "TACHYON_BUILD_WORKLOAD_SPEC_JSON")]
        spec_env: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ListBuildsResponse {
    pub(super) builds: Vec<BuildResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct BuildResponse {
    pub(super) id: String,
    pub(super) app_id: String,
    #[serde(default)]
    pub(super) trigger: Option<String>,
    #[serde(default)]
    pub(super) source_branch: Option<String>,
    #[serde(default)]
    pub(super) commit_sha: Option<String>,
    #[serde(default)]
    pub(super) commit_message: Option<String>,
    #[serde(default)]
    pub(super) pr_number: Option<i32>,
    pub(super) status: String,
    #[serde(default)]
    pub(super) duration_secs: Option<i32>,
    #[serde(default)]
    pub(super) error_message: Option<String>,
    #[serde(default)]
    pub(super) created_at: Option<String>,
    #[serde(default)]
    pub(super) updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct BuildLogsResponse {
    pub(super) lines: Vec<BuildLogLineResponse>,
    pub(super) next_token: Option<String>,
    pub(super) is_complete: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct BuildLogLineResponse {
    pub(super) timestamp: i64,
    pub(super) message: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum AgentBuildEvent<'a> {
    Build {
        build_id: &'a str,
        status: &'a str,
    },
    Log {
        build_id: &'a str,
        timestamp: i64,
        message: String,
    },
    Result {
        build_id: &'a str,
        status: &'a str,
        exit_code: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_message: Option<&'a str>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct TriggerBuildRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pr_number: Option<i32>,
}

#[derive(Debug, Serialize)]
struct CreatePreviewBuildRequest {
    source_branch: String,
}

pub(super) async fn run_builds_list(
    api: &ApiClient,
    app_id: &str,
    limit: usize,
    json: bool,
) -> Result<()> {
    let resp: ListBuildsResponse = api
        .get(&format!("/v1/compute/apps/{app_id}/builds"))
        .await?;
    if json {
        let builds = &resp.builds[..resp.builds.len().min(limit)];
        return print_json(&builds);
    }
    if resp.builds.is_empty() {
        println!("No builds found for app {app_id}");
        return Ok(());
    }
    let builds = &resp.builds[..resp.builds.len().min(limit)];
    println!(
        "{:<26}  {:<11}  {:<20}  {:<8}  {:<19}",
        "BUILD ID", "STATUS", "BRANCH", "COMMIT", "CREATED AT"
    );
    println!(
        "{:-<26}  {:-<11}  {:-<20}  {:-<8}  {:-<19}",
        "", "", "", "", ""
    );
    for build in builds {
        println!(
            "{:<26}  {:<11}  {:<20}  {:<8}  {:<19}",
            build.id,
            build.status,
            truncate(build.source_branch.as_deref().unwrap_or("-"), 20),
            truncate_sha(build.commit_sha.as_deref().unwrap_or("-")),
            build
                .created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }

    // Show active preview URLs below the build list.
    let build_pr_numbers: HashMap<&str, i32> = builds
        .iter()
        .filter_map(|build| build.pr_number.map(|pr| (build.id.as_str(), pr)))
        .collect();
    let preview_url = format!("/v1/compute/apps/{app_id}/deployments?environment=preview");
    if let Ok(dep_resp) = api.get::<ListDeploymentsResponse>(&preview_url).await {
        let active: Vec<_> = dep_resp
            .deployments
            .iter()
            .filter(|d| d.status == "active" || d.status == "deploying")
            .collect();
        if !active.is_empty() {
            println!();
            println!("Preview URLs:");
            for dep in &active {
                let branch = dep.source_branch.as_deref().unwrap_or("-");
                let pr_number = dep.pr_number.or_else(|| {
                    dep.build_id
                        .as_deref()
                        .and_then(|build_id| build_pr_numbers.get(build_id).copied())
                });
                println!("  [{branch}] {}", dep.display_url_with_pr(pr_number));
            }
        }
    }
    Ok(())
}

pub(super) async fn run_builds_get(api: &ApiClient, build_id: &str, json: bool) -> Result<()> {
    let build: BuildResponse = api.get(&format!("/v1/compute/builds/{build_id}")).await?;
    if json {
        return print_json(&build);
    }
    println!("ID:       {}", build.id);
    println!("App ID:   {}", build.app_id);
    println!("Status:   {}", build.status);
    println!(
        "Branch:   {}",
        build.source_branch.as_deref().unwrap_or("-")
    );
    println!("Commit:   {}", build.commit_sha.as_deref().unwrap_or("-"));
    println!(
        "Message:  {}",
        build.commit_message.as_deref().unwrap_or("-")
    );
    println!("Trigger:  {}", build.trigger.as_deref().unwrap_or("-"));
    if let Some(dur) = build.duration_secs {
        println!("Duration: {dur}s");
    }
    if let Some(err) = &build.error_message {
        println!("Error:    {err}");
    }
    println!(
        "Created:  {}",
        build
            .created_at
            .as_deref()
            .map(format_created_at)
            .unwrap_or_else(|| "-".to_string())
    );

    // Fetch the associated deployment to show the preview URL.
    if build.status == "succeeded" {
        let url: String = format!("/v1/compute/apps/{}/deployments", build.app_id);
        if let Ok(resp) = api.get::<ListDeploymentsResponse>(&url).await {
            if let Some(dep) = resp
                .deployments
                .iter()
                .find(|d| d.build_id.as_deref() == Some(&build.id))
            {
                let pr_number = dep.pr_number.or(build.pr_number);
                println!("Preview:  {}", dep.display_url_with_pr(pr_number));
            }
        }
    }
    Ok(())
}

fn git_output(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| anyhow!("failed to run git: {e}"))?;
    if !output.status.success() {
        return Err(anyhow!("`git {}` failed", args.join(" ")));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn detect_current_git_branch() -> Result<String> {
    let branch = git_output(&["rev-parse", "--abbrev-ref", "HEAD"])
        .map_err(|e| anyhow!("{e}; not inside a git repository? Pass --branch explicitly."))?;
    if branch.is_empty() || branch == "HEAD" {
        return Err(anyhow!("detached HEAD state; pass --branch explicitly"));
    }
    Ok(branch)
}

/// Extract `(owner, repo)` from a GitHub remote URL
/// (`git@github.com:owner/repo.git` or `https://github.com/owner/repo`).
pub(super) fn parse_github_remote(url: &str) -> Option<(String, String)> {
    let rest = url
        .strip_prefix("git@github.com:")
        .or_else(|| url.strip_prefix("ssh://git@github.com/"))
        .or_else(|| url.strip_prefix("https://github.com/"))
        .or_else(|| url.strip_prefix("http://github.com/"))?;
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let mut parts = rest.splitn(2, '/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim().trim_end_matches('/');
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// Find cloud apps whose connected repository matches the
/// `origin` remote of the current git repository. A repo can
/// back multiple apps (e.g. two frontends), so all matches are
/// returned.
async fn resolve_apps_by_git_remote(api: &ApiClient) -> Result<Vec<AppResponse>> {
    let remote_url = git_output(&["remote", "get-url", "origin"]).map_err(|e| {
        anyhow!(
            "{e}; could not auto-detect the app. Pass an app name, \
             set metadata.name in tachyon.yml, or run inside a git \
             repository with an `origin` remote."
        )
    })?;
    let (owner, repo) = parse_github_remote(&remote_url)
        .ok_or_else(|| anyhow!("unsupported git remote URL: {remote_url}"))?;

    let resp: ListAppsResponse = api.get("/v1/compute/apps").await?;
    let apps: Vec<AppResponse> = resp
        .apps
        .into_iter()
        .filter(|app| {
            app.repository_owner.as_deref() == Some(owner.as_str())
                && app.repository_name.as_deref() == Some(repo.as_str())
        })
        .collect();

    if apps.is_empty() {
        return Err(anyhow!(
            "no cloud apps found for repository {owner}/{repo} in the \
             current tenant (check `tachyon switch` / `tachyon compute apps list`)"
        ));
    }
    Ok(apps)
}

pub(super) async fn run_preview(
    api: &ApiClient,
    app_id: &Option<String>,
    project_config: Option<&ProjectConfig>,
    branch: Option<&str>,
    pr: Option<i32>,
) -> Result<()> {
    let branch_name = match branch {
        Some(b) => b.to_string(),
        None => detect_current_git_branch()?,
    };
    println!("Branch: {branch_name}");

    // App resolution: explicit arg > tachyon.yml > git remote match.
    let targets: Vec<(String, String)> = match app_id_or_default(app_id, project_config) {
        Ok(app) => {
            let id = resolve::resolve_app_id(api, app).await?;
            vec![(id, app.to_string())]
        }
        Err(_) => resolve_apps_by_git_remote(api)
            .await?
            .into_iter()
            .map(|app| (app.id, app.name))
            .collect(),
    };

    for (id, name) in &targets {
        println!("App: {name}");
        run_builds_trigger(api, id, Some(&branch_name), pr).await?;
    }
    Ok(())
}

pub(super) async fn run_preview_create(api: &ApiClient, app: &str, branch: &str) -> Result<()> {
    let app_id = resolve::resolve_app_id(api, app).await?;
    let req = CreatePreviewBuildRequest {
        source_branch: branch.to_string(),
    };
    let build: BuildResponse = api.post(&format!("/v1/apps/{app_id}/builds"), &req).await?;

    println!("Preview build created: {}", build.id);
    println!("Status: {}", build.status);
    if let Some(branch) = build.source_branch.as_deref() {
        println!("Branch: {branch}");
    }
    if let Some(pr_number) = build.pr_number {
        println!("PR: #{pr_number}");
    }
    Ok(())
}

pub(super) async fn run_builds_trigger(
    api: &ApiClient,
    app_id: &str,
    branch: Option<&str>,
    pr: Option<i32>,
) -> Result<()> {
    let req = TriggerBuildRequest {
        branch: branch.map(String::from),
        pr_number: pr,
    };
    let build: BuildResponse = api
        .post(&format!("/v1/compute/apps/{app_id}/builds"), &req)
        .await?;
    println!("Build triggered: {}", build.id);
    println!("Status: {}", build.status);
    if let Some(branch) = build.source_branch.as_deref() {
        println!("Branch: {branch}");
    }
    if let Some(pr_number) = build.pr_number {
        println!("PR: #{pr_number} (preview build)");
    }
    Ok(())
}

pub(super) async fn run_builds_cancel(api: &ApiClient, build_id: &str) -> Result<()> {
    api.post_no_body(&format!("/v1/compute/builds/{build_id}/cancel"))
        .await?;
    println!("Build {build_id} cancelled.");
    Ok(())
}

pub(super) async fn run_builds_logs(
    api: &ApiClient,
    app_id: Option<&str>,
    build_id: Option<&str>,
    follow: bool,
    agent: bool,
) -> Result<()> {
    let resolved_build_id = match build_id {
        Some(id) => id.to_string(),
        None => {
            let app_id = app_id
                .ok_or_else(|| anyhow!("app_id required when --build-id is not specified"))?;
            let resp: ListBuildsResponse = api
                .get(&format!("/v1/compute/apps/{app_id}/builds"))
                .await?;
            resp.builds
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("no builds found for app {app_id}"))?
                .id
        }
    };

    let mut next_token: Option<String> = None;
    let mut is_complete = false;
    let mut last_none_token_signature: Option<Vec<(i64, String)>> = None;
    let mut no_progress_none_token_polls = 0_usize;
    let follow_interval = build_logs_follow_interval();
    loop {
        let path = format!("/v1/compute/builds/{resolved_build_id}/logs");
        let logs: BuildLogsResponse = if let Some(token) = &next_token {
            api.get_query(&path, &[("next_token", token.as_str())])
                .await?
        } else {
            api.get(&path).await?
        };

        let response_next_token = logs.next_token.clone();
        let none_token_signature = response_next_token
            .is_none()
            .then(|| build_log_lines_signature(&logs.lines));
        let no_progress_none_token_response = follow
            && !logs.is_complete
            && response_next_token.is_none()
            && none_token_signature.as_ref().is_some_and(|signature| {
                signature.is_empty() || last_none_token_signature.as_ref() == Some(signature)
            });

        if !no_progress_none_token_response {
            for line in &logs.lines {
                if agent {
                    print_agent_event(&AgentBuildEvent::Log {
                        build_id: &resolved_build_id,
                        timestamp: line.timestamp,
                        message: compact_agent_message(&line.message),
                    })?;
                } else {
                    println!("[{}] {}", format_timestamp_ms(line.timestamp), line.message);
                }
            }
        }

        if logs.is_complete {
            is_complete = true;
            break;
        }

        if let Some(signature) = none_token_signature {
            last_none_token_signature = Some(signature);
        } else {
            last_none_token_signature = None;
        }
        if no_progress_none_token_response {
            no_progress_none_token_polls += 1;
        } else {
            no_progress_none_token_polls = 0;
        }

        next_token = response_next_token;
        if follow {
            if no_progress_none_token_polls >= BUILD_LOGS_MAX_NO_PROGRESS_NONE_TOKEN_POLLS {
                break;
            }
            sleep(follow_interval).await;
        } else {
            break;
        }
    }

    if follow && is_complete {
        let build: BuildResponse = api
            .get(&format!("/v1/compute/builds/{resolved_build_id}"))
            .await?;
        if agent {
            let exit_code = if is_success_build_status(&build.status) {
                0
            } else {
                1
            };
            print_agent_event(&AgentBuildEvent::Result {
                build_id: &resolved_build_id,
                status: &build.status,
                exit_code,
                error_message: build.error_message.as_deref(),
            })?;
        }
        if !is_success_build_status(&build.status) {
            return Err(anyhow!("build {} failed", resolved_build_id));
        }
    }

    Ok(())
}

fn build_log_lines_signature(lines: &[BuildLogLineResponse]) -> Vec<(i64, String)> {
    lines
        .iter()
        .map(|line| (line.timestamp, line.message.clone()))
        .collect()
}

fn build_logs_follow_interval() -> Duration {
    std::env::var("TACHYON_COMPUTE_BUILD_LOGS_FOLLOW_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(BUILD_LOGS_FOLLOW_INTERVAL)
}

pub(super) async fn run_builds_watch(
    api: &ApiClient,
    app_id: Option<&str>,
    build_id: Option<&str>,
    interval_secs: u64,
    timeout_secs: Option<u64>,
    no_logs: bool,
    agent: bool,
) -> Result<()> {
    let resolved_build_id = match build_id {
        Some(id) => id.to_string(),
        None => {
            let app_id = app_id
                .ok_or_else(|| anyhow!("app_id required when --build-id is not specified"))?;
            let resp: ListBuildsResponse = api
                .get(&format!("/v1/compute/apps/{app_id}/builds"))
                .await?;
            resp.builds
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("no builds found for app {app_id}"))?
                .id
        }
    };

    let interval = Duration::from_secs(interval_secs.max(1));
    let started = Instant::now();
    let timeout = timeout_secs.map(Duration::from_secs);
    let mut next_token: Option<String> = None;
    let mut last_status: Option<String> = None;

    loop {
        if let Some(timeout) = timeout {
            if started.elapsed() >= timeout {
                if agent {
                    print_agent_event(&AgentBuildEvent::Result {
                        build_id: &resolved_build_id,
                        status: "timeout",
                        exit_code: 124,
                        error_message: Some("watch timed out"),
                    })?;
                }
                return Err(anyhow!("build {} watch timed out", resolved_build_id));
            }
        }

        let build: BuildResponse = api
            .get(&format!("/v1/compute/builds/{resolved_build_id}"))
            .await?;
        if last_status.as_deref() != Some(build.status.as_str()) {
            if agent {
                print_agent_event(&AgentBuildEvent::Build {
                    build_id: &resolved_build_id,
                    status: &build.status,
                })?;
            } else {
                println!("Build {}: {}", resolved_build_id, build.status);
            }
            last_status = Some(build.status.clone());
        }

        if !no_logs {
            let path = format!("/v1/compute/builds/{resolved_build_id}/logs");
            let logs_result: Result<BuildLogsResponse> = if let Some(token) = &next_token {
                api.get_query(&path, &[("next_token", token.as_str())])
                    .await
            } else {
                api.get(&path).await
            };
            let logs = match logs_result {
                Ok(logs) => logs,
                Err(err)
                    if !is_terminal_build_status(&build.status)
                        && is_http_not_found_error(&err) =>
                {
                    sleep(interval).await;
                    continue;
                }
                Err(err) => return Err(err),
            };
            for line in &logs.lines {
                if agent {
                    print_agent_event(&AgentBuildEvent::Log {
                        build_id: &resolved_build_id,
                        timestamp: line.timestamp,
                        message: compact_agent_message(&line.message),
                    })?;
                } else {
                    println!("[{}] {}", format_timestamp_ms(line.timestamp), line.message);
                }
            }
            next_token = logs.next_token;
        }

        if is_terminal_build_status(&build.status) {
            let exit_code = if is_success_build_status(&build.status) {
                0
            } else {
                1
            };
            if agent {
                print_agent_event(&AgentBuildEvent::Result {
                    build_id: &resolved_build_id,
                    status: &build.status,
                    exit_code,
                    error_message: build.error_message.as_deref(),
                })?;
            } else if exit_code == 0 {
                println!("Build {} completed successfully.", resolved_build_id);
            }
            if exit_code != 0 {
                return Err(anyhow!(
                    "build {} finished with status {}",
                    resolved_build_id,
                    build.status
                ));
            }
            return Ok(());
        }

        sleep(interval).await;
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeLogTailOptions {
    pub(super) raw_json: bool,
}

#[derive(Debug, Default)]
pub(super) struct SseEvent {
    pub(super) event: Option<String>,
    pub(super) data: Vec<String>,
}

pub(super) async fn run_runtime_log_tail(
    api: &ApiClient,
    app_id: &str,
    options: RuntimeLogTailOptions,
) -> Result<()> {
    let url = format!("{}/v1/compute/apps/{app_id}/logs/tail", api.base_url);
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        let result = tokio::select! {
            biased;
            signal = tokio::signal::ctrl_c() => {
                signal?;
                eprintln!("runtime log tail stopped.");
                return Ok(());
            }
            result = run_runtime_log_tail_once(api, &url, options) => result,
        };

        if let Err(error) = result {
            eprintln!("runtime log tail error: {error}");
        } else {
            eprintln!("runtime log tail disconnected.");
        }

        eprintln!("reconnecting in {}s...", backoff.as_secs());
        tokio::select! {
            biased;
            signal = tokio::signal::ctrl_c() => {
                signal?;
                eprintln!("runtime log tail stopped.");
                return Ok(());
            }
            _ = sleep(backoff) => {}
        }
        backoff = std::cmp::min(backoff * 2, max_backoff);
    }
}

pub(super) async fn run_runtime_log_tail_once(
    api: &ApiClient,
    url: &str,
    options: RuntimeLogTailOptions,
) -> Result<()> {
    let mut resp = api
        .client
        .get(url)
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .send()
        .await
        .map_err(|e| anyhow!("GET {url} failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "runtime log tail failed: status={status}, body={body}"
        ));
    }

    let mut utf8 = Utf8ChunkBuffer::default();
    let mut pending = String::new();
    while let Some(chunk) = resp.chunk().await? {
        pending.push_str(&utf8.push_chunk(&chunk)?);
        for event in drain_sse_events(&mut pending) {
            if print_runtime_log_tail_event(&event, options.raw_json)? {
                return Err(anyhow!("runtime log tail stream error"));
            }
        }
        std::io::stdout().flush().ok();
    }
    pending.push_str(&utf8.finish()?);
    if let Some(event) = parse_sse_event(&pending) {
        if print_runtime_log_tail_event(&event, options.raw_json)? {
            return Err(anyhow!("runtime log tail stream error"));
        }
        std::io::stdout().flush().ok();
    }

    Ok(())
}

#[derive(Debug, Default)]
pub(super) struct Utf8ChunkBuffer {
    pending: Vec<u8>,
}

impl Utf8ChunkBuffer {
    pub(super) fn push_chunk(&mut self, chunk: &[u8]) -> Result<String> {
        self.pending.extend_from_slice(chunk);
        let valid_len = match std::str::from_utf8(&self.pending) {
            Ok(_) => self.pending.len(),
            Err(error) if error.error_len().is_none() => error.valid_up_to(),
            Err(error) => {
                return Err(anyhow!(
                    "runtime log tail received invalid UTF-8 at byte {}",
                    error.valid_up_to()
                ));
            }
        };
        if valid_len == 0 {
            return Ok(String::new());
        }
        let text = std::str::from_utf8(&self.pending[..valid_len])
            .expect("valid_up_to must point at a UTF-8 boundary")
            .to_string();
        self.pending.drain(..valid_len);
        Ok(text)
    }

    pub(super) fn finish(self) -> Result<String> {
        if self.pending.is_empty() {
            return Ok(String::new());
        }
        String::from_utf8(self.pending)
            .map_err(|error| anyhow!("runtime log tail ended with incomplete UTF-8: {error}"))
    }
}

pub(super) fn drain_sse_events(pending: &mut String) -> Vec<SseEvent> {
    let mut events = Vec::new();
    while let Some((pos, delimiter_len)) = find_sse_event_boundary(pending) {
        let raw = pending[..pos].to_string();
        pending.drain(..pos + delimiter_len);
        if let Some(event) = parse_sse_event(&raw) {
            events.push(event);
        }
    }
    events
}

fn find_sse_event_boundary(input: &str) -> Option<(usize, usize)> {
    match (input.find("\r\n\r\n"), input.find("\n\n")) {
        (Some(crlf), Some(lf)) if crlf <= lf => Some((crlf, 4)),
        (Some(_), Some(lf)) => Some((lf, 2)),
        (Some(crlf), None) => Some((crlf, 4)),
        (None, Some(lf)) => Some((lf, 2)),
        (None, None) => None,
    }
}

fn parse_sse_event(raw: &str) -> Option<SseEvent> {
    let mut event = SseEvent::default();
    for line in raw.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        if let Some(value) = line.strip_prefix("event:") {
            event.event = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            event.data.push(value.trim_start().to_string());
        }
    }
    (!event.data.is_empty()).then_some(event)
}

fn print_runtime_log_tail_event(event: &SseEvent, raw_json: bool) -> Result<bool> {
    let data = event.data.join("\n");
    if event.event.as_deref() == Some("error") {
        eprintln!(
            "runtime log tail stream error: {}",
            format_runtime_log_line(&data)
        );
        return Ok(true);
    }

    if raw_json {
        println!("{data}");
    } else {
        for line in format_runtime_log_lines(&data) {
            println!("{line}");
        }
    }
    Ok(false)
}

pub(super) fn format_runtime_log_lines(data: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(data) else {
        return vec![data.to_string()];
    };
    let Some(object) = value.as_object() else {
        return vec![format_runtime_log_value(&value)];
    };

    let timestamp = runtime_log_timestamp(object);
    let mut lines = Vec::new();

    if let Some(summary) = runtime_log_request_summary(object) {
        lines.push(format!("{timestamp} {summary}"));
    }

    if let Some(exceptions) = object.get("exceptions").and_then(Value::as_array) {
        for exception in exceptions {
            lines.push(format!(
                "{timestamp} ERROR {}",
                format_runtime_log_exception(exception)
            ));
        }
    }

    if let Some(logs) = object.get("logs").and_then(Value::as_array) {
        for log in logs {
            lines.push(format!("{timestamp} {}", format_runtime_log_console(log)));
        }
    }

    if lines.is_empty() {
        lines.push(format!(
            "{timestamp} {}",
            runtime_log_generic_summary(object, &value)
        ));
    }

    lines
}

fn format_runtime_log_line(data: &str) -> String {
    format_runtime_log_lines(data).join(" | ")
}

fn runtime_log_timestamp(object: &serde_json::Map<String, Value>) -> String {
    let millis = object
        .get("eventTimestamp")
        .or_else(|| object.get("timestamp"))
        .and_then(value_to_i64);
    if let Some(millis) = millis {
        return match Utc.timestamp_millis_opt(millis) {
            chrono::LocalResult::Single(dt) => format!("[{}]", dt.format("%H:%M:%S")),
            _ => format!("[{millis}]"),
        };
    }

    let timestamp = object
        .get("timestamp")
        .or_else(|| object.get("eventTimestamp"))
        .and_then(Value::as_str);
    if let Some(timestamp) = timestamp {
        if let Ok(parsed) = DateTime::parse_from_rfc3339(timestamp) {
            return format!("[{}]", parsed.with_timezone(&Utc).format("%H:%M:%S"));
        }
    }

    format!("[{}]", Utc::now().format("%H:%M:%S"))
}

fn runtime_log_request_summary(object: &serde_json::Map<String, Value>) -> Option<String> {
    let request = object.get("request")?.as_object()?;
    let method = request
        .get("method")
        .or_else(|| request.get("requestMethod"))
        .and_then(Value::as_str)
        .unwrap_or("REQUEST");
    let url = request
        .get("url")
        .or_else(|| request.get("path"))
        .and_then(Value::as_str)
        .unwrap_or("-");
    let path = runtime_log_url_path(url);
    let status = object
        .get("response")
        .and_then(Value::as_object)
        .and_then(|response| {
            response
                .get("status")
                .or_else(|| response.get("statusCode"))
                .and_then(value_to_i64)
        })
        .map(|status| status.to_string())
        .unwrap_or_else(|| "-".to_string());
    let cpu = object
        .get("cpuTime")
        .or_else(|| object.get("cpu_time"))
        .and_then(value_to_i64);
    let source = object
        .get("scriptName")
        .or_else(|| object.get("source"))
        .and_then(Value::as_str);

    let mut line = format!("{method} {path} {status}");
    if let Some(cpu) = cpu {
        line.push_str(&format!("  (cpu: {cpu}ms)"));
    }
    if let Some(source) = source {
        line.push_str(&format!("  source={source}"));
    }
    Some(line)
}

fn runtime_log_generic_summary(object: &serde_json::Map<String, Value>, value: &Value) -> String {
    let level = object
        .get("level")
        .or_else(|| object.get("severity"))
        .and_then(Value::as_str)
        .map(|level| level.to_ascii_uppercase());
    let source = object
        .get("source")
        .or_else(|| object.get("scriptName"))
        .and_then(Value::as_str);
    let message = runtime_log_message(object).unwrap_or_else(|| format_runtime_log_value(value));

    let mut parts = Vec::new();
    if let Some(level) = level {
        parts.push(level);
    }
    parts.push(message);
    if let Some(source) = source {
        parts.push(format!("source={source}"));
    }
    parts.join(" ")
}

fn runtime_log_message(object: &serde_json::Map<String, Value>) -> Option<String> {
    for key in ["message", "outcome", "event"] {
        if let Some(value) = object.get(key) {
            let text = format_runtime_log_value(value);
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

fn runtime_log_url_path(url: &str) -> String {
    if let Ok(parsed) = reqwest::Url::parse(url) {
        let mut path = parsed.path().to_string();
        if let Some(query) = parsed.query() {
            path.push('?');
            path.push_str(query);
        }
        if path.is_empty() {
            "/".to_string()
        } else {
            path
        }
    } else {
        url.to_string()
    }
}

fn format_runtime_log_console(value: &Value) -> String {
    if let Some(object) = value.as_object() {
        let level = object
            .get("level")
            .or_else(|| object.get("severity"))
            .and_then(Value::as_str)
            .map(|level| level.to_ascii_uppercase());
        let message = object
            .get("message")
            .or_else(|| object.get("text"))
            .or_else(|| object.get("args"))
            .map(format_runtime_log_value)
            .unwrap_or_else(|| format_runtime_log_value(value));
        if let Some(level) = level {
            return format!("{level} {message}");
        }
        return message;
    }
    format_runtime_log_value(value)
}

fn format_runtime_log_exception(value: &Value) -> String {
    if let Some(object) = value.as_object() {
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Exception");
        let message = object
            .get("message")
            .or_else(|| object.get("text"))
            .map(format_runtime_log_value)
            .unwrap_or_else(|| format_runtime_log_value(value));
        if message.starts_with(name) {
            message
        } else {
            format!("{name}: {message}")
        }
    } else {
        format_runtime_log_value(value)
    }
}

fn format_runtime_log_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .map(format_runtime_log_value)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        _ => serde_json::to_string(value).unwrap_or_else(|_| value.to_string()),
    }
}

fn value_to_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .or_else(|| value.as_f64().map(|value| value as i64))
}

/// Reproduce a Cloud Apps build locally in Docker. See `build_reproduce` for
/// the parsing/invocation logic; this function wires CLI args and exit codes.
pub(super) fn run_builds_reproduce(
    build_id: &str,
    mock_path: Option<&std::path::Path>,
    source_dir: Option<&std::path::Path>,
    image_override: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let mock_path = mock_path.ok_or_else(|| {
        anyhow!(
            "Phase 1: --mock <path> is required. \
             The build-config endpoint (PLT-913) is not yet available; \
             pass a local YAML/JSON fixture matching BuildConfig."
        )
    })?;

    let config = build_reproduce::load_mock_config(mock_path)?;
    let spec = build_reproduce::BuildSpec::parse(&config.buildspec)?;

    let owned_cwd;
    let source_dir: &std::path::Path = match source_dir {
        Some(p) => p,
        None => {
            owned_cwd = std::env::current_dir()?;
            owned_cwd.as_path()
        }
    };

    let invocation = build_reproduce::build_invocation(&config, &spec, source_dir, image_override);

    eprintln!("=== reproduce build {} ===", build_id);
    eprintln!("  config build_id: {}", config.build_id);
    eprintln!("  image:           {}", invocation.image);
    eprintln!("  source:          {}", invocation.source_dir.display());
    if !config.environment.secret_names.is_empty() {
        eprintln!(
            "  secrets (names): {}",
            config.environment.secret_names.join(", ")
        );
        eprintln!("  (secret values are not provided in mock mode)");
    }
    eprintln!();

    if dry_run {
        println!("{}", invocation.to_display_string());
        return Ok(());
    }

    let exit_code = invocation.execute()?;
    if exit_code != 0 {
        return Err(anyhow!("reproduce build exited with code {exit_code}"));
    }
    Ok(())
}
