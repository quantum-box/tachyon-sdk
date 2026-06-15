use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::process::Stdio;
use std::time::Duration;
use tachyon_sdk::apis::configuration::Configuration;
use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::Message;

use crate::auth::StoredCredentials;
use crate::client::ApiClient;

const SERVICE_NAME: &str = "tachyon-worker";
const UNIT_PATH: &str = "/etc/systemd/system/tachyon-worker.service";
const ENV_DIR: &str = "/etc/tachyon";
const ENV_PATH: &str = "/etc/tachyon/worker.env";
const DEFAULT_PROVIDER: &str = "containerized_codex";
const TOOL_JOB_WORKER_POLICY_ID: &str = "pol_01tooljobworkerpolicy";

#[derive(Args)]
pub struct WorkerArgs {
    #[command(subcommand)]
    pub command: WorkerCommand,
}

#[derive(Subcommand)]
pub enum WorkerCommand {
    /// Install and start the local worker as a systemd service
    Start(StartArgs),
    /// Show the local worker systemd status
    Status,
    /// Show local worker logs from journald
    Logs(LogsArgs),
    /// Stop the local worker systemd service
    Stop,
    /// Restart the local worker systemd service
    Restart,
    /// Run the worker foreground process used by systemd
    Run(RunArgs),
}

#[derive(Args)]
pub struct StartArgs {
    /// Stable worker ID. Defaults to worker-<hostname>.
    #[arg(long, env = "TACHYON_WORKER_ID")]
    pub worker_id: Option<String>,

    /// Provider capability to advertise.
    #[arg(long, env = "TACHYON_WORKER_PROVIDER", default_value_t = WorkerProvider::ContainerizedCodex)]
    pub provider: WorkerProvider,

    /// Maximum concurrent jobs advertised to Tachyon Cloud.
    #[arg(long, env = "TACHYON_WORKER_MAX_CONCURRENT_JOBS", default_value_t = 1)]
    pub max_concurrent_jobs: u32,

    /// Skip the automatic self-update before installing the service.
    #[arg(long)]
    pub no_update: bool,

    /// Print the files and commands that would be used without mutating the host.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct RunArgs {
    /// Stable worker ID. Defaults to worker-<hostname>.
    #[arg(long, env = "TACHYON_WORKER_ID")]
    pub worker_id: Option<String>,

    /// Provider capability to advertise.
    #[arg(long, env = "TACHYON_WORKER_PROVIDER", default_value_t = WorkerProvider::ContainerizedCodex)]
    pub provider: WorkerProvider,

    /// Maximum concurrent jobs advertised to Tachyon Cloud.
    #[arg(long, env = "TACHYON_WORKER_MAX_CONCURRENT_JOBS", default_value_t = 1)]
    pub max_concurrent_jobs: u32,

    /// Poll interval used when no job is available.
    #[arg(long, env = "TACHYON_WORKER_POLL_INTERVAL_MS", default_value_t = 1000)]
    pub poll_interval_ms: u64,
}

#[derive(Args)]
pub struct LogsArgs {
    /// Number of recent log lines to show.
    #[arg(short = 'n', long, default_value_t = 100)]
    pub lines: u32,

    /// Follow logs.
    #[arg(short, long)]
    pub follow: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum WorkerProvider {
    #[value(name = "containerized_codex", alias = "containerized-codex")]
    ContainerizedCodex,
}

impl std::fmt::Display for WorkerProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerProvider::ContainerizedCodex => write!(f, "{DEFAULT_PROVIDER}"),
        }
    }
}

pub async fn run(
    args: &WorkerArgs,
    config: &Configuration,
    tenant_id: &str,
    profile: &str,
) -> Result<()> {
    match &args.command {
        WorkerCommand::Start(start_args) => start(start_args, config, tenant_id, profile).await,
        WorkerCommand::Status => systemctl_status(),
        WorkerCommand::Logs(logs_args) => journalctl_logs(logs_args),
        WorkerCommand::Stop => systemctl_simple("stop"),
        WorkerCommand::Restart => systemctl_simple("restart"),
        WorkerCommand::Run(run_args) => run_foreground(run_args, config, tenant_id).await,
    }
}

async fn start(
    args: &StartArgs,
    config: &Configuration,
    tenant_id: &str,
    profile: &str,
) -> Result<()> {
    if !cfg!(target_os = "linux") && !args.dry_run {
        bail!("tachyon worker start currently supports Linux systemd hosts only");
    }
    if !args.dry_run && current_uid()? != 0 {
        bail!("tachyon worker start must be run with sudo so it can install a systemd service");
    }
    if !args.no_update && !args.dry_run {
        crate::install_cli::run()
            .await
            .context("failed to self-update tachyon before installing worker")?;
    }

    let worker_id = args.worker_id.clone().unwrap_or_else(default_worker_id);
    let binary = current_binary_path()?;
    let service_user = service_user();
    let home = home_for_user(&service_user).unwrap_or_else(|| PathBuf::from("/root"));
    let service_profile = load_service_profile_credentials(&home, profile);
    let effective_profile = service_profile
        .as_ref()
        .map(|(name, _)| name.as_str())
        .unwrap_or(profile);
    let effective_tenant_id = if tenant_id.is_empty() {
        service_profile
            .as_ref()
            .and_then(|(_, creds)| creds.operator_id.clone())
            .unwrap_or_default()
    } else {
        tenant_id.to_string()
    };

    if effective_tenant_id.is_empty() {
        bail!("operator id is not configured. Run `tachyon login` or pass `--tenant-id`");
    }
    let install_config = config_for_worker_install(config, service_profile.as_ref());
    if install_config.bearer_access_token.is_none() {
        bail!("authentication is not configured. Run `tachyon login` before `sudo tachyon worker start`");
    }

    let worker_api_key = if args.dry_run {
        None
    } else {
        Some(create_worker_api_key(&install_config, &effective_tenant_id, &worker_id).await?)
    };

    let env_content = render_env_file(&WorkerEnvFile {
        config: &install_config,
        tenant_id: &effective_tenant_id,
        profile: effective_profile,
        worker_id: &worker_id,
        provider: &args.provider,
        max_concurrent_jobs: args.max_concurrent_jobs,
        home: &home,
        worker_api_key: worker_api_key.as_deref(),
    });
    let unit_content = render_unit_file(&binary, &service_user);

    if args.dry_run {
        println!("Would write {ENV_PATH}:\n{env_content}");
        println!("Would write {UNIT_PATH}:\n{unit_content}");
        println!("Would create a dedicated worker API key.");
        println!("Would run: systemctl daemon-reload");
        println!("Would run: systemctl enable --now {SERVICE_NAME}");
        return Ok(());
    }

    fs::create_dir_all(ENV_DIR).context("failed to create /etc/tachyon")?;
    fs::write(ENV_PATH, env_content).context("failed to write worker env file")?;
    set_owner_readable_only(ENV_PATH)?;
    fs::write(UNIT_PATH, unit_content).context("failed to write systemd unit")?;

    run_command("systemctl", &["daemon-reload"])?;
    run_command("systemctl", &["enable", "--now", SERVICE_NAME])?;

    println!("Tachyon worker installed and started.");
    println!("worker_id: {worker_id}");
    println!("provider: {}", args.provider);
    println!("service: {SERVICE_NAME}");
    Ok(())
}

async fn create_worker_api_key(
    config: &Configuration,
    tenant_id: &str,
    worker_id: &str,
) -> Result<String> {
    let api = ApiClient::new(config, tenant_id)?;
    let request = CreateOrgApiKeyRequest {
        name: format!("{worker_id}-key"),
        service_account_id: None,
        service_account_name: Some(format!("{SERVICE_NAME}-{worker_id}")),
        ttl_seconds: None,
        policy_ids: vec![TOOL_JOB_WORKER_POLICY_ID.to_string()],
    };
    let response: ApiKeyResponse = api
        .post(&format!("/v1/orgs/{tenant_id}/api-keys"), &request)
        .await
        .context("failed to create worker API key")?;
    Ok(response.value)
}

fn config_for_worker_install(
    config: &Configuration,
    service_profile: Option<&(String, StoredCredentials)>,
) -> Configuration {
    let mut install_config = config.clone();
    if install_config.bearer_access_token.is_none() {
        if let Some((_, credentials)) = service_profile {
            install_config.bearer_access_token = Some(credentials.access_token.clone());
        }
    }
    install_config
}

fn systemctl_simple(action: &str) -> Result<()> {
    run_command("systemctl", &[action, SERVICE_NAME])
}

fn systemctl_status() -> Result<()> {
    run_command("systemctl", &["status", SERVICE_NAME, "--no-pager"])
}

fn journalctl_logs(args: &LogsArgs) -> Result<()> {
    let lines = args.lines.to_string();
    let mut command_args = vec!["-u", SERVICE_NAME, "-n", &lines, "--no-pager"];
    if args.follow {
        command_args.push("-f");
    }
    run_command("journalctl", &command_args)
}

async fn run_foreground(args: &RunArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    if tenant_id.is_empty() {
        bail!("operator id is not configured. Set TACHYON_TENANT_ID or pass --tenant-id");
    }
    if config.bearer_access_token.is_none() {
        bail!("authentication is not configured. Run `tachyon login` or set TACHYON_API_KEY");
    }

    let api = ApiClient::new(config, tenant_id)?;
    let worker_id = args.worker_id.clone().unwrap_or_else(default_worker_id);
    let register = RegisterWorkerRequest {
        worker_id: Some(worker_id.clone()),
        name: Some(worker_id.clone()),
        hostname: hostname(),
        capabilities: vec![args.provider.to_string()],
        repo_path: None,
        queue_type: None,
        queue_url: None,
        max_concurrent_jobs: Some(args.max_concurrent_jobs),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        system_info: Some(system_info()),
    };
    let response: RegisterWorkerResponse = api
        .post("/v1/agent/workers/register", &register)
        .await
        .context("failed to register worker")?;

    let registered_id = response.worker.worker_id;
    println!("Tachyon worker registered: {registered_id}");
    println!("provider: {}", args.provider);

    loop {
        let active_jobs = if claim_and_process_job(&api, &registered_id, &args.provider).await? {
            0
        } else {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(args.poll_interval_ms)) => {}
                _ = shutdown_signal() => {
                    let _ = api.post_no_body(&format!("/v1/agent/workers/{registered_id}/deregister")).await;
                    println!("Tachyon worker stopped: {registered_id}");
                    return Ok(());
                }
            }
            0
        };

        let heartbeat = WorkerHeartbeatRequest {
            active_jobs,
            status: Some("idle".to_string()),
            system_metrics: Some(system_metrics()),
        };
        let response: WorkerHeartbeatResponse = api
            .post(
                &format!("/v1/agent/workers/{registered_id}/heartbeat"),
                &heartbeat,
            )
            .await
            .context("failed to send worker heartbeat")?;
        let interval = response.next_heartbeat_seconds.max(10);

        let sleep_ms = args.poll_interval_ms.min(interval as u64 * 1000);
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(sleep_ms)) => {}
            _ = shutdown_signal() => {
                let _ = api.post_no_body(&format!("/v1/agent/workers/{registered_id}/deregister")).await;
                println!("Tachyon worker stopped: {registered_id}");
                return Ok(());
            }
        }
    }
}

async fn claim_and_process_job(
    api: &ApiClient,
    worker_id: &str,
    provider: &WorkerProvider,
) -> Result<bool> {
    let response: ClaimWorkerToolJobResponse = api
        .post(
            "/v1/agent/tool-jobs/claim",
            &ClaimWorkerToolJobRequest {
                worker_id: worker_id.to_string(),
                provider: provider.to_string(),
            },
        )
        .await
        .context("failed to claim worker Tool Job")?;

    let Some(job) = response.job else {
        return Ok(false);
    };

    println!("Claimed Tool Job: {}", job.id);
    let started_at = chrono::Utc::now();
    let result = run_containerized_codex(&job).await;
    let completed_at = chrono::Utc::now();

    let completion = match result {
        Ok(output) => CompleteWorkerToolJobRequest {
            worker_id: worker_id.to_string(),
            status: "SUCCEEDED".to_string(),
            result_json: Some(json!({ "output": output.result_text })),
            raw_events: output.raw_events,
            artifacts: Vec::new(),
            exit_code: output.exit_code,
            error_message: None,
            estimated_nanodollars: 0,
            observed_nanodollars: 0,
            started_at: Some(started_at.to_rfc3339()),
            completed_at: Some(completed_at.to_rfc3339()),
            session_id: output.session_id,
            worktree_used: false,
            worktree_info: None,
            pr_url: None,
            has_conflicts: false,
            conflicts: Vec::new(),
        },
        Err(error) => CompleteWorkerToolJobRequest {
            worker_id: worker_id.to_string(),
            status: "FAILED".to_string(),
            result_json: None,
            raw_events: vec![json!({
                "event_type": "error",
                "payload": { "message": error.to_string() }
            })],
            artifacts: Vec::new(),
            exit_code: None,
            error_message: Some(error.to_string()),
            estimated_nanodollars: 0,
            observed_nanodollars: 0,
            started_at: Some(started_at.to_rfc3339()),
            completed_at: Some(completed_at.to_rfc3339()),
            session_id: None,
            worktree_used: false,
            worktree_info: None,
            pr_url: None,
            has_conflicts: false,
            conflicts: Vec::new(),
        },
    };

    api.post::<_, serde_json::Value>(
        &format!("/v1/agent/tool-jobs/{}/worker-complete", job.id),
        &completion,
    )
    .await
    .with_context(|| format!("failed to complete Tool Job {}", job.id))?;

    println!("Completed Tool Job: {}", job.id);
    Ok(true)
}

async fn run_containerized_codex(job: &WorkerToolJob) -> Result<WorkerJobOutput> {
    ensure_docker_available().await?;

    if job.execution_mode() == WorkerExecutionMode::AppServerWs {
        return run_containerized_codex_app_server(job).await;
    }

    let container_name = format!("tachyon-codex-{}", job.id.to_lowercase());
    let docker_args = build_codex_exec_docker_args(job, &container_name);

    let output = TokioCommand::new("docker")
        .args(&docker_args)
        .kill_on_drop(true)
        .output()
        .await
        .context("failed to run containerized Codex")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let mut raw_events = Vec::new();
    if !stdout.trim().is_empty() {
        raw_events.push(json!({
            "event_type": "stdout",
            "payload": { "text": stdout }
        }));
    }
    if !stderr.trim().is_empty() {
        raw_events.push(json!({
            "event_type": "stderr",
            "payload": { "text": stderr }
        }));
    }

    if !output.status.success() {
        bail!("Codex container exited with non-zero status: {stderr}");
    }

    let result_text = stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|value| {
            value
                .get("output")
                .and_then(|output| output.as_str())
                .map(ToOwned::to_owned)
        })
        .next_back()
        .unwrap_or(stdout);

    Ok(WorkerJobOutput {
        result_text,
        raw_events,
        exit_code: output.status.code(),
        session_id: None,
    })
}

async fn run_containerized_codex_app_server(job: &WorkerToolJob) -> Result<WorkerJobOutput> {
    let port = allocate_loopback_port()?;
    let listen_url = format!("ws://127.0.0.1:{port}");
    let container_name = format!("tachyon-codex-{}", job.id.to_lowercase());
    let docker_args = build_codex_app_server_docker_args(job, &container_name, &listen_url);
    let timeout_secs = job.timeout_seconds();

    let mut child = TokioCommand::new("docker")
        .args(&docker_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .context("failed to run containerized Codex app-server")?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture Codex app-server stdout"))?;
    let stdout_reader = tokio::spawn(async move {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf).await;
        String::from_utf8_lossy(&buf).to_string()
    });

    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("failed to capture Codex app-server stderr"))?;
    let stderr_reader = tokio::spawn(async move {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf).await;
        String::from_utf8_lossy(&buf).to_string()
    });

    let invocation = CodexAppServerInvocation::from_job(job);
    let execution_result = timeout(
        Duration::from_secs(timeout_secs),
        run_app_server_connection(&listen_url, invocation),
    )
    .await;

    remove_container(&container_name).await;

    let status = timeout(Duration::from_secs(5), child.wait())
        .await
        .ok()
        .and_then(|result| result.ok());
    let stdout = timeout(Duration::from_secs(5), stdout_reader)
        .await
        .ok()
        .and_then(|result| result.ok())
        .unwrap_or_default();
    let stderr = timeout(Duration::from_secs(5), stderr_reader)
        .await
        .ok()
        .and_then(|result| result.ok())
        .unwrap_or_default();

    let execution = execution_result
        .with_context(|| format!("Codex app-server timed out after {timeout_secs}s"))??;
    let result_text = extract_agent_text(&execution.events).unwrap_or_default();
    let mut raw_events = execution
        .events
        .into_iter()
        .map(codex_event_to_raw_event)
        .collect::<Vec<_>>();
    if !stdout.trim().is_empty() {
        raw_events.push(text_raw_event("stdout", stdout));
    }
    if !stderr.trim().is_empty() {
        raw_events.push(text_raw_event("stderr", stderr));
    }
    if let Some(status) = status {
        raw_events.push(json!({
            "event_type": "container.exit",
            "payload": { "exit_code": status.code() }
        }));
    }

    Ok(WorkerJobOutput {
        result_text,
        raw_events,
        exit_code: Some(0),
        session_id: execution.thread_id,
    })
}

fn build_codex_exec_docker_args(job: &WorkerToolJob, container_name: &str) -> Vec<String> {
    let image = job.container_image();
    let network = env::var("CODEX_CONTAINER_NETWORK").unwrap_or_else(|_| "bridge".to_string());
    let memory = env::var("CODEX_CONTAINER_MEMORY").unwrap_or_else(|_| "2g".to_string());
    let workspace = job.host_workspace();

    let mut docker_args = base_docker_args(job, container_name, &network, &workspace, &memory);
    docker_args.extend([
        image,
        "codex".to_string(),
        "exec".to_string(),
        "--json".to_string(),
        job.prompt.clone(),
    ]);
    docker_args
}

fn build_codex_app_server_docker_args(
    job: &WorkerToolJob,
    container_name: &str,
    listen_url: &str,
) -> Vec<String> {
    let image = job.container_image();
    let network = job.app_server_network();
    let memory = job.container_memory();
    let workspace = job.host_workspace();

    let mut docker_args = base_docker_args(job, container_name, &network, &workspace, &memory);
    docker_args.extend([
        image,
        "codex".to_string(),
        "app-server".to_string(),
        "--listen".to_string(),
        listen_url.to_string(),
    ]);
    docker_args
}

fn base_docker_args(
    job: &WorkerToolJob,
    container_name: &str,
    network: &str,
    workspace: &str,
    memory: &str,
) -> Vec<String> {
    let mut docker_args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "--name".to_string(),
        container_name.to_string(),
        "--label".to_string(),
        "tachyon.worker.managed=true".to_string(),
        "--label".to_string(),
        format!("tachyon.tool_job_id={}", job.id),
        format!("--network={network}"),
        "-v".to_string(),
        format!("{workspace}:/workspace"),
        "-w".to_string(),
        job.container_working_dir(),
        "--memory".to_string(),
        memory.to_string(),
    ];

    if env::var_os("OPENAI_API_KEY").is_some() {
        docker_args.extend(["-e".to_string(), "OPENAI_API_KEY".to_string()]);
    }

    if let Some(codex_home) = job.codex_home() {
        docker_args.extend(["-v".to_string(), format!("{codex_home}:/codex-home")]);
        docker_args.extend(["-e".to_string(), "CODEX_HOME=/codex-home".to_string()]);
    }

    let mut environment = job.environment.iter().collect::<Vec<_>>();
    environment.sort_by(|a, b| a.0.cmp(b.0));
    for (key, value) in environment {
        docker_args.extend(["-e".to_string(), format!("{key}={value}")]);
    }

    docker_args
}

async fn run_app_server_connection(
    listen_url: &str,
    invocation: CodexAppServerInvocation,
) -> Result<CodexAppServerExecution> {
    let (mut ws, _) = connect_with_retry(listen_url).await?;
    let mut events = Vec::new();

    send_request(
        &mut ws,
        1,
        "initialize",
        json!({
            "clientInfo": {
                "name": invocation.service_name,
                "title": "Tachyon Worker",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": {
                "experimentalApi": invocation.experimental_api,
            }
        }),
    )
    .await?;
    read_until_response(&mut ws, 1, &mut events).await?;
    send_notification(&mut ws, "initialized").await?;

    let mut thread_params = invocation.thread_params();
    let thread_method = if let Some(thread_id) = &invocation.resume_session_id {
        thread_params.insert("threadId".to_string(), json!(thread_id));
        "thread/resume"
    } else {
        "thread/start"
    };
    send_request(&mut ws, 2, thread_method, Value::Object(thread_params)).await?;
    let thread_result = read_until_response(&mut ws, 2, &mut events).await?;
    let thread_id = thread_result
        .get("thread")
        .and_then(|thread| thread.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Codex app-server response did not include thread.id"))?;

    send_request(
        &mut ws,
        3,
        "turn/start",
        Value::Object(invocation.turn_params(&thread_id)),
    )
    .await?;
    let turn_result = read_until_response(&mut ws, 3, &mut events).await?;
    let turn_id = turn_result
        .get("turn")
        .and_then(|turn| turn.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string);

    while let Some(message) = ws.next().await {
        let message = message.context("Codex app-server websocket read failed")?;
        if let Some(event) = decode_notification(message)? {
            let completed = event.method == "turn/completed";
            let failed = event.method == "error";
            events.push(event);
            if completed {
                return Ok(CodexAppServerExecution {
                    thread_id: Some(thread_id),
                    turn_id,
                    events,
                });
            }
            if failed {
                bail!("Codex app-server turn failed");
            }
        }
    }

    bail!("Codex app-server websocket closed before turn/completed")
}

async fn connect_with_retry(
    listen_url: &str,
) -> Result<(
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    http::Response<Option<Vec<u8>>>,
)> {
    let mut last_error = None;
    for _ in 0..200 {
        match tokio_tungstenite::connect_async(listen_url).await {
            Ok(connection) => return Ok(connection),
            Err(err) => {
                last_error = Some(err.to_string());
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    Err(anyhow!(
        "failed to connect to Codex app-server websocket: {}",
        last_error.unwrap_or_else(|| "unknown error".to_string())
    ))
}

async fn send_request<S>(ws: &mut S, id: u64, method: &str, params: Value) -> Result<()>
where
    S: futures_util::Sink<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::fmt::Display,
{
    send_json(ws, json!({ "method": method, "id": id, "params": params })).await
}

async fn send_notification<S>(ws: &mut S, method: &str) -> Result<()>
where
    S: futures_util::Sink<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::fmt::Display,
{
    send_json(ws, json!({ "method": method })).await
}

async fn send_json<S>(ws: &mut S, value: Value) -> Result<()>
where
    S: futures_util::Sink<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::fmt::Display,
{
    ws.send(Message::Text(value.to_string().into()))
        .await
        .map_err(|err| anyhow!("Codex app-server websocket send failed: {err}"))
}

async fn read_until_response<S>(
    ws: &mut S,
    expected_id: u64,
    events: &mut Vec<CodexAppServerEvent>,
) -> Result<Value>
where
    S: StreamExt<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Unpin,
{
    while let Some(message) = ws.next().await {
        let message = message.context("Codex app-server websocket read failed")?;
        let Some(text) = message.into_text().ok() else {
            continue;
        };
        let rpc: RpcMessage =
            serde_json::from_str(&text).context("invalid Codex app-server JSON-RPC message")?;

        if rpc.id == Some(expected_id) {
            if let Some(error) = rpc.error {
                bail!(
                    "Codex app-server request failed: {} ({})",
                    error.message,
                    error.code
                );
            }
            return Ok(rpc.result.unwrap_or(Value::Null));
        }

        if let Some(method) = rpc.method {
            events.push(CodexAppServerEvent {
                method,
                params: rpc.params,
            });
        }
    }

    bail!("Codex app-server websocket closed while waiting for response")
}

fn decode_notification(message: Message) -> Result<Option<CodexAppServerEvent>> {
    let Some(text) = message.into_text().ok() else {
        return Ok(None);
    };
    let rpc: RpcMessage =
        serde_json::from_str(&text).context("invalid Codex app-server JSON-RPC notification")?;
    Ok(rpc.method.map(|method| CodexAppServerEvent {
        method,
        params: rpc.params,
    }))
}

fn allocate_loopback_port() -> Result<u16> {
    let listener =
        TcpListener::bind("127.0.0.1:0").context("failed to allocate Codex app-server port")?;
    let port = listener
        .local_addr()
        .context("failed to read Codex app-server port")?
        .port();
    drop(listener);
    Ok(port)
}

async fn remove_container(name: &str) {
    let _ = TokioCommand::new("docker")
        .args(["rm", "-f", name])
        .output()
        .await;
}

async fn ensure_docker_available() -> Result<()> {
    let output = TokioCommand::new("docker")
        .args(["version", "--format", "{{.Server.Version}}"])
        .output()
        .await
        .context("Docker CLI is not available")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Docker daemon is not available: {stderr}");
    }
    Ok(())
}

#[derive(Serialize)]
struct RegisterWorkerRequest {
    worker_id: Option<String>,
    name: Option<String>,
    hostname: Option<String>,
    capabilities: Vec<String>,
    repo_path: Option<String>,
    queue_type: Option<String>,
    queue_url: Option<String>,
    max_concurrent_jobs: Option<u32>,
    version: Option<String>,
    system_info: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct RegisterWorkerResponse {
    worker: RegisteredWorker,
}

#[derive(Deserialize)]
struct RegisteredWorker {
    worker_id: String,
}

#[derive(Serialize)]
struct WorkerHeartbeatRequest {
    active_jobs: u32,
    status: Option<String>,
    system_metrics: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct WorkerHeartbeatResponse {
    next_heartbeat_seconds: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateOrgApiKeyRequest {
    name: String,
    service_account_id: Option<String>,
    service_account_name: Option<String>,
    ttl_seconds: Option<i64>,
    policy_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiKeyResponse {
    value: String,
}

#[derive(Serialize)]
struct ClaimWorkerToolJobRequest {
    worker_id: String,
    provider: String,
}

#[derive(Deserialize)]
struct ClaimWorkerToolJobResponse {
    job: Option<WorkerToolJob>,
}

#[derive(Deserialize)]
struct WorkerToolJob {
    id: String,
    prompt: String,
    #[serde(default)]
    output_profile: Option<String>,
    #[serde(default)]
    context_paths: Vec<String>,
    #[serde(default)]
    environment: HashMap<String, String>,
    #[serde(default)]
    metadata: Value,
    #[serde(default)]
    container_execution: Option<WorkerContainerExecutionConfig>,
    #[serde(default)]
    resume_session_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerExecutionMode {
    Exec,
    AppServerWs,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct WorkerContainerExecutionConfig {
    codex_mode: Option<String>,
    image: Option<String>,
    workspace: Option<String>,
    working_dir: Option<String>,
    cwd: Option<String>,
    codex_home: Option<String>,
    network: Option<String>,
    resource_limits: Option<WorkerContainerResourceLimitsConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct WorkerContainerResourceLimitsConfig {
    memory: Option<String>,
}

impl WorkerToolJob {
    fn execution_mode(&self) -> WorkerExecutionMode {
        let mode = self
            .container_execution
            .as_ref()
            .and_then(|config| config.codex_mode.as_deref())
            .or_else(|| metadata_string(&self.metadata, &["codex_mode"]))
            .or_else(|| metadata_string(&self.metadata, &["container", "codex_mode"]));

        match mode {
            Some("app_server_ws" | "codex_app_server_ws") => WorkerExecutionMode::AppServerWs,
            _ => WorkerExecutionMode::Exec,
        }
    }

    fn container_image(&self) -> String {
        self.container_execution
            .as_ref()
            .and_then(|config| config.image.clone())
            .or_else(|| {
                metadata_string(&self.metadata, &["container", "image"]).map(str::to_string)
            })
            .or_else(|| env::var("CODEX_CONTAINER_IMAGE").ok())
            .unwrap_or_else(|| "ghcr.io/quantum-box/codex-runner:latest".to_string())
    }

    fn container_memory(&self) -> String {
        self.container_execution
            .as_ref()
            .and_then(|config| config.resource_limits.as_ref())
            .and_then(|limits| limits.memory.clone())
            .or_else(|| {
                metadata_string(&self.metadata, &["container", "resource_limits", "memory"])
                    .map(str::to_string)
            })
            .or_else(|| {
                metadata_string(&self.metadata, &["container", "resources", "memory"])
                    .map(str::to_string)
            })
            .or_else(|| env::var("CODEX_CONTAINER_MEMORY").ok())
            .unwrap_or_else(|| "2g".to_string())
    }

    fn host_workspace(&self) -> String {
        self.container_execution
            .as_ref()
            .and_then(|config| config.workspace.clone())
            .or_else(|| {
                metadata_string(&self.metadata, &["container", "workspace"]).map(str::to_string)
            })
            .filter(|path| Path::new(path).exists())
            .or_else(|| {
                self.context_paths
                    .iter()
                    .find(|path| Path::new(path.as_str()).exists())
                    .cloned()
            })
            .unwrap_or_else(|| "/tmp".to_string())
    }

    fn container_working_dir(&self) -> String {
        self.container_execution
            .as_ref()
            .and_then(|config| config.working_dir.clone())
            .or_else(|| {
                metadata_string(&self.metadata, &["container", "working_dir"]).map(str::to_string)
            })
            .unwrap_or_else(|| "/workspace".to_string())
    }

    fn invocation_cwd(&self) -> String {
        self.container_execution
            .as_ref()
            .and_then(|config| config.cwd.clone())
            .or_else(|| metadata_string(&self.metadata, &["cwd"]).map(str::to_string))
            .unwrap_or_else(|| self.container_working_dir())
    }

    fn app_server_network(&self) -> String {
        let requested = self
            .container_execution
            .as_ref()
            .and_then(|config| config.network.as_deref())
            .or_else(|| metadata_string(&self.metadata, &["container", "network"]))
            .or_else(|| metadata_string(&self.metadata, &["network"]))
            .unwrap_or("host_loopback");

        match requested {
            "host" | "host_loopback" => "host".to_string(),
            _ => requested.to_string(),
        }
    }

    fn codex_home(&self) -> Option<String> {
        self.container_execution
            .as_ref()
            .and_then(|config| config.codex_home.clone())
            .or_else(|| {
                metadata_string(&self.metadata, &["container", "codex_home"]).map(str::to_string)
            })
            .or_else(|| env::var("CODEX_HOME").ok())
            .or_else(|| env::var("HOME").ok().map(|home| format!("{home}/.codex")))
            .filter(|path| Path::new(path).exists())
    }

    fn timeout_seconds(&self) -> u64 {
        metadata_u64(&self.metadata, &["timeout_seconds"])
            .or_else(|| metadata_u64(&self.metadata, &["timeout_sec"]))
            .unwrap_or(1800)
    }
}

#[derive(Serialize)]
struct CompleteWorkerToolJobRequest {
    worker_id: String,
    status: String,
    result_json: Option<serde_json::Value>,
    raw_events: Vec<serde_json::Value>,
    artifacts: Vec<serde_json::Value>,
    exit_code: Option<i32>,
    error_message: Option<String>,
    estimated_nanodollars: i64,
    observed_nanodollars: i64,
    started_at: Option<String>,
    completed_at: Option<String>,
    session_id: Option<String>,
    worktree_used: bool,
    worktree_info: Option<serde_json::Value>,
    pr_url: Option<String>,
    has_conflicts: bool,
    conflicts: Vec<String>,
}

struct WorkerJobOutput {
    result_text: String,
    raw_events: Vec<serde_json::Value>,
    exit_code: Option<i32>,
    session_id: Option<String>,
}

struct CodexAppServerInvocation {
    prompt: String,
    resume_session_id: Option<String>,
    model: Option<String>,
    cwd: String,
    approval_policy: Option<String>,
    sandbox_policy: Option<Value>,
    effort: Option<String>,
    summary: Option<String>,
    personality: Option<String>,
    output_schema: Option<Value>,
    service_name: String,
    experimental_api: bool,
}

impl CodexAppServerInvocation {
    fn from_job(job: &WorkerToolJob) -> Self {
        Self {
            prompt: job.prompt.clone(),
            resume_session_id: job.resume_session_id.clone(),
            model: metadata_string(&job.metadata, &["model"])
                .map(str::to_string)
                .or_else(|| job.output_profile.clone()),
            cwd: job.invocation_cwd(),
            approval_policy: metadata_string(&job.metadata, &["approval_policy"])
                .map(str::to_string)
                .or_else(|| Some("never".to_string())),
            sandbox_policy: metadata_value(&job.metadata, &["sandbox_policy"]).cloned(),
            effort: metadata_string(&job.metadata, &["effort"]).map(str::to_string),
            summary: metadata_string(&job.metadata, &["summary"]).map(str::to_string),
            personality: metadata_string(&job.metadata, &["personality"]).map(str::to_string),
            output_schema: metadata_value(&job.metadata, &["output_schema"]).cloned(),
            service_name: metadata_string(&job.metadata, &["service_name"])
                .unwrap_or("tachyon_cli_worker")
                .to_string(),
            experimental_api: metadata_bool(&job.metadata, &["experimental_api"]).unwrap_or(false),
        }
    }

    fn thread_params(&self) -> Map<String, Value> {
        let mut params = Map::new();
        insert_string(&mut params, "model", self.model.as_deref());
        insert_string(&mut params, "cwd", Some(&self.cwd));
        insert_string(
            &mut params,
            "approvalPolicy",
            self.approval_policy.as_deref(),
        );
        insert_string(&mut params, "personality", self.personality.as_deref());
        insert_string(&mut params, "serviceName", Some(&self.service_name));
        params
    }

    fn turn_params(&self, thread_id: &str) -> Map<String, Value> {
        let mut params = Map::new();
        params.insert("threadId".to_string(), json!(thread_id));
        params.insert(
            "input".to_string(),
            json!([{ "type": "text", "text": self.prompt, "text_elements": [] }]),
        );
        insert_string(&mut params, "model", self.model.as_deref());
        insert_string(&mut params, "cwd", Some(&self.cwd));
        insert_string(
            &mut params,
            "approvalPolicy",
            self.approval_policy.as_deref(),
        );
        insert_string(&mut params, "effort", self.effort.as_deref());
        insert_string(&mut params, "summary", self.summary.as_deref());
        insert_string(&mut params, "personality", self.personality.as_deref());
        if let Some(sandbox_policy) = &self.sandbox_policy {
            params.insert("sandboxPolicy".to_string(), sandbox_policy.clone());
        }
        if let Some(output_schema) = &self.output_schema {
            params.insert("outputSchema".to_string(), output_schema.clone());
        }
        params
    }
}

struct CodexAppServerEvent {
    method: String,
    params: Option<Value>,
}

struct CodexAppServerExecution {
    thread_id: Option<String>,
    #[allow(dead_code)]
    turn_id: Option<String>,
    events: Vec<CodexAppServerEvent>,
}

#[derive(Deserialize)]
struct RpcMessage {
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Option<Value>,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

struct WorkerEnvFile<'a> {
    config: &'a Configuration,
    tenant_id: &'a str,
    profile: &'a str,
    worker_id: &'a str,
    provider: &'a WorkerProvider,
    max_concurrent_jobs: u32,
    home: &'a Path,
    worker_api_key: Option<&'a str>,
}

fn render_env_file(env: &WorkerEnvFile<'_>) -> String {
    let mut lines = vec![
        env_line("TACHYON_API_URL", &env.config.base_path),
        env_line("TACHYON_TENANT_ID", env.tenant_id),
        env_line("TACHYON_PROFILE", env.profile),
        env_line("TACHYON_WORKER_ID", env.worker_id),
        env_line("TACHYON_WORKER_PROVIDER", &env.provider.to_string()),
        env_line(
            "TACHYON_WORKER_MAX_CONCURRENT_JOBS",
            &env.max_concurrent_jobs.to_string(),
        ),
        env_line("HOME", &env.home.to_string_lossy()),
        env_line(
            "XDG_CONFIG_HOME",
            &env.home.join(".config").to_string_lossy(),
        ),
    ];

    if let Some(worker_api_key) = env.worker_api_key {
        lines.push(env_line("TACHYON_API_KEY", worker_api_key));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn render_unit_file(binary: &Path, service_user: &str) -> String {
    let user_line = if service_user == "root" {
        String::new()
    } else {
        format!("User={service_user}\n")
    };
    format!(
        "[Unit]\n\
         Description=Tachyon Worker\n\
         Wants=network-online.target docker.service\n\
         After=network-online.target docker.service\n\
         \n\
         [Service]\n\
         Type=simple\n\
         {user_line}EnvironmentFile={ENV_PATH}\n\
         ExecStart={} worker run\n\
         Restart=always\n\
         RestartSec=5\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n",
        binary.display()
    )
}

fn metadata_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn metadata_string<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    metadata_value(value, path).and_then(Value::as_str)
}

fn metadata_u64(value: &Value, path: &[&str]) -> Option<u64> {
    metadata_value(value, path).and_then(Value::as_u64)
}

fn metadata_bool(value: &Value, path: &[&str]) -> Option<bool> {
    metadata_value(value, path).and_then(Value::as_bool)
}

fn insert_string(params: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        params.insert(key.to_string(), json!(value));
    }
}

fn codex_event_to_raw_event(event: CodexAppServerEvent) -> Value {
    let event_name = match event.method.as_str() {
        "thread/started" => "thread.started".to_string(),
        "thread/status/changed" => "thread.status_changed".to_string(),
        "turn/started" => "turn.started".to_string(),
        "turn/completed" => "turn.completed".to_string(),
        "item/started" => "item.started".to_string(),
        "item/completed" => "item.completed".to_string(),
        "item/agentMessage/delta" => "item.agent_message.delta".to_string(),
        other => other.replace('/', "."),
    };

    json!({
        "event_type": format!("codex.{event_name}"),
        "payload": event.params,
    })
}

fn text_raw_event(event_type: &str, text: impl Into<String>) -> Value {
    json!({
        "event_type": event_type,
        "payload": { "text": text.into() },
    })
}

fn extract_agent_text(events: &[CodexAppServerEvent]) -> Option<String> {
    let mut delta_text = String::new();
    let mut completed_text = None;

    for event in events {
        let Some(params) = event.params.as_ref() else {
            continue;
        };
        match event.method.as_str() {
            "item/agentMessage/delta" => {
                if let Some(delta) = params
                    .get("delta")
                    .or_else(|| params.get("text"))
                    .and_then(Value::as_str)
                {
                    delta_text.push_str(delta);
                }
            }
            "item/completed" => {
                if let Some(text) = params
                    .get("item")
                    .and_then(|item| item.get("text"))
                    .or_else(|| params.get("text"))
                    .and_then(Value::as_str)
                {
                    completed_text = Some(text.to_string());
                }
            }
            _ => {}
        }
    }

    completed_text.or({
        if delta_text.is_empty() {
            None
        } else {
            Some(delta_text)
        }
    })
}

fn env_line(key: &str, value: &str) -> String {
    format!("{key}={}", quote_env_value(value))
}

fn quote_env_value(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn default_worker_id() -> String {
    let host = hostname().unwrap_or_else(|| "local".to_string());
    let sanitized: String = host
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("worker-{sanitized}")
}

fn hostname() -> Option<String> {
    StdCommand::new("hostname")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
}

fn system_info() -> serde_json::Value {
    json!({
        "os": env::consts::OS,
        "arch": env::consts::ARCH,
        "hostname": hostname(),
        "version": env!("CARGO_PKG_VERSION"),
    })
}

fn system_metrics() -> serde_json::Value {
    json!({
        "heartbeat_source": "tachyon-cli",
        "version": env!("CARGO_PKG_VERSION"),
    })
}

fn current_binary_path() -> Result<PathBuf> {
    env::current_exe().context("failed to determine tachyon binary path")
}

fn current_uid() -> Result<u32> {
    let output = StdCommand::new("id")
        .arg("-u")
        .output()
        .context("failed to run id -u")?;
    if !output.status.success() {
        bail!("id -u failed");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .trim()
        .parse::<u32>()
        .context("failed to parse current uid")
}

fn service_user() -> String {
    env::var("SUDO_USER")
        .ok()
        .filter(|user| !user.is_empty() && user != "root")
        .unwrap_or_else(|| "root".to_string())
}

fn home_for_user(user: &str) -> Option<PathBuf> {
    if user == "root" {
        return Some(PathBuf::from("/root"));
    }
    let passwd = fs::read_to_string("/etc/passwd").ok()?;
    passwd.lines().find_map(|line| {
        let mut parts = line.split(':');
        let name = parts.next()?;
        if name != user {
            return None;
        }
        let fields: Vec<&str> = parts.collect();
        fields.get(4).map(PathBuf::from)
    })
}

fn load_service_profile_credentials(
    home: &Path,
    requested_profile: &str,
) -> Option<(String, StoredCredentials)> {
    let config_dir = home.join(".config").join("tachyon");
    let mut candidates = vec![requested_profile.to_string()];

    if let Ok(active) = fs::read_to_string(config_dir.join("active_profile")) {
        let active = active.trim();
        if !active.is_empty() && !candidates.iter().any(|candidate| candidate == active) {
            candidates.push(active.to_string());
        }
    }

    if !candidates
        .iter()
        .any(|candidate| candidate == crate::auth::DEFAULT_PROFILE)
    {
        candidates.push(crate::auth::DEFAULT_PROFILE.to_string());
    }

    for profile in candidates {
        let path = config_dir.join("profiles").join(format!("{profile}.json"));
        let Ok(contents) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(credentials) = serde_json::from_str::<StoredCredentials>(&contents) else {
            continue;
        };
        return Some((profile, credentials));
    }

    None
}

fn set_owner_readable_only(path: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("failed to set permissions on {path}"))?;
    }
    Ok(())
}

fn run_command(program: &str, args: &[&str]) -> Result<()> {
    let status = StdCommand::new(program)
        .args(args)
        .status()
        .with_context(|| format!("failed to run {program} {}", args.join(" ")))?;
    if !status.success() {
        return Err(anyhow!(
            "{} {} failed with status {}",
            program,
            args.join(" "),
            status
        ));
    }
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let terminate = async {
            if let Ok(mut signal) = signal(SignalKind::terminate()) {
                signal.recv().await;
            }
        };

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn render_unit_uses_tachyon_worker_run() {
        let unit = render_unit_file(Path::new("/usr/local/bin/tachyon"), "tachyon");

        assert!(unit.contains("User=tachyon"));
        assert!(unit.contains("EnvironmentFile=/etc/tachyon/worker.env"));
        assert!(unit.contains("ExecStart=/usr/local/bin/tachyon worker run"));
        assert!(unit.contains("Restart=always"));
    }

    #[test]
    fn render_env_file_writes_worker_context() {
        let config = Configuration {
            base_path: "https://api.n1.tachy.one".to_string(),
            bearer_access_token: None,
            ..Configuration::default()
        };

        let env = render_env_file(&WorkerEnvFile {
            config: &config,
            tenant_id: "tn_test",
            profile: "work",
            worker_id: "worker-test",
            provider: &WorkerProvider::ContainerizedCodex,
            max_concurrent_jobs: 2,
            home: Path::new("/home/tachyon"),
            worker_api_key: Some("pk_test"),
        });

        assert!(env.contains("TACHYON_API_URL='https://api.n1.tachy.one'"));
        assert!(env.contains("TACHYON_TENANT_ID='tn_test'"));
        assert!(env.contains("TACHYON_PROFILE='work'"));
        assert!(env.contains("TACHYON_WORKER_ID='worker-test'"));
        assert!(env.contains("TACHYON_WORKER_PROVIDER='containerized_codex'"));
        assert!(env.contains("TACHYON_WORKER_MAX_CONCURRENT_JOBS='2'"));
        assert!(env.contains("TACHYON_API_KEY='pk_test'"));
    }

    #[test]
    fn deserializes_claimed_job_app_server_metadata() {
        let response: ClaimWorkerToolJobResponse = serde_json::from_value(json!({
            "job": {
                "id": "01KVTESTAPPSERVER000000000",
                "prompt": "inspect",
                "context_paths": ["/tmp"],
                "output_profile": "gpt-5.4-mini",
                "environment": { "A_VAR": "a" },
                "metadata": {
                    "codex_mode": "app_server_ws",
                    "cwd": "/workspace",
                    "model": "gpt-5.4-mini"
                },
                "container_execution": {
                    "codex_mode": "app_server_ws",
                    "working_dir": "/workspace",
                    "network": "host_loopback"
                },
                "resume_session_id": "thread_existing"
            }
        }))
        .unwrap();

        let job = response.job.unwrap();
        assert_eq!(job.execution_mode(), WorkerExecutionMode::AppServerWs);
        assert_eq!(job.output_profile.as_deref(), Some("gpt-5.4-mini"));
        assert_eq!(job.resume_session_id.as_deref(), Some("thread_existing"));
        assert_eq!(job.environment.get("A_VAR").map(String::as_str), Some("a"));
    }

    #[test]
    fn app_server_mode_builds_app_server_docker_args() {
        let workspace = TempDir::new().unwrap();
        let job = WorkerToolJob {
            id: "01KVTESTAPPSERVER000000000".to_string(),
            prompt: "inspect".to_string(),
            output_profile: Some("gpt-5.4-mini".to_string()),
            context_paths: vec![workspace.path().display().to_string()],
            environment: HashMap::from([("Z_VAR".to_string(), "z".to_string())]),
            metadata: json!({
                "codex_mode": "app_server_ws",
                "cwd": "/workspace",
                "approval_policy": "never",
                "timeout_seconds": 30
            }),
            container_execution: Some(WorkerContainerExecutionConfig {
                codex_mode: Some("app_server_ws".to_string()),
                image: Some("codex:test".to_string()),
                workspace: Some(workspace.path().display().to_string()),
                working_dir: Some("/workspace".to_string()),
                cwd: Some("/workspace".to_string()),
                codex_home: None,
                network: Some("host_loopback".to_string()),
                resource_limits: Some(WorkerContainerResourceLimitsConfig {
                    memory: Some("1g".to_string()),
                }),
            }),
            resume_session_id: None,
        };

        let args =
            build_codex_app_server_docker_args(&job, "tachyon-codex-test", "ws://127.0.0.1:34567");
        let image_index = args.iter().position(|arg| arg == "codex:test").unwrap();

        assert_eq!(job.execution_mode(), WorkerExecutionMode::AppServerWs);
        assert!(args.contains(&"--network=host".to_string()));
        assert!(args.contains(&format!("{}:/workspace", workspace.path().display())));
        assert_eq!(
            &args[image_index + 1..],
            &[
                "codex".to_string(),
                "app-server".to_string(),
                "--listen".to_string(),
                "ws://127.0.0.1:34567".to_string(),
            ]
        );
        assert!(!args
            .windows(3)
            .any(|window| window == ["codex", "exec", "--json"]));
    }

    #[test]
    fn default_mode_keeps_codex_exec_args() {
        let workspace = TempDir::new().unwrap();
        let job = WorkerToolJob {
            id: "01KVTESTEXEC000000000000".to_string(),
            prompt: "inspect".to_string(),
            output_profile: None,
            context_paths: vec![workspace.path().display().to_string()],
            environment: HashMap::new(),
            metadata: Value::Null,
            container_execution: None,
            resume_session_id: None,
        };

        let args = build_codex_exec_docker_args(&job, "tachyon-codex-test");
        let image_index = args
            .iter()
            .position(|arg| arg == "ghcr.io/quantum-box/codex-runner:latest")
            .unwrap();

        assert_eq!(job.execution_mode(), WorkerExecutionMode::Exec);
        assert_eq!(
            &args[image_index + 1..],
            &[
                "codex".to_string(),
                "exec".to_string(),
                "--json".to_string(),
                "inspect".to_string(),
            ]
        );
    }
}
