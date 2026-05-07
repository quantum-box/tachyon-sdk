use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, truncate, ApiClient};
use crate::resolve;

#[derive(Debug, Clone, Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AgentCommand {
    /// Manage agent sessions
    #[command(visible_alias = "session")]
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },
    /// Manage agent protocols
    Protocols {
        #[command(subcommand)]
        command: ProtocolsCommand,
    },
    /// Manage agent workers
    Workers {
        #[command(subcommand)]
        command: WorkersCommand,
    },
    /// Manage agent worktrees
    Worktrees {
        #[command(subcommand)]
        command: WorktreesCommand,
    },
    /// Manage agent memory
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    /// Check agent status
    Status {
        /// Agent ID
        agent_id: String,
        /// Session ID (optional)
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Get agent messages
    Messages {
        /// Agent ID
        agent_id: String,
        /// Session ID (optional)
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List available LLM models
    Models {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum SessionsCommand {
    /// List agent sessions
    List {
        #[arg(long)]
        json: bool,
    },
    /// Diagnose an agent session from persisted events and related Tool Jobs
    Diagnose {
        /// Session ID or legacy chatroom ID
        session_id: String,
        #[arg(long)]
        json: bool,
        /// Return a non-zero exit code for the selected condition.
        #[arg(long, value_enum)]
        fail_on: Option<FailOn>,
    },
    /// Print raw structured event timeline for an agent session
    Events {
        /// Session ID or legacy chatroom ID
        session_id: String,
        /// Maximum number of recent events to return
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FailOn {
    Anomaly,
    Error,
    Incomplete,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProtocolsCommand {
    /// List agent protocols
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get agent protocol details
    Get {
        /// Protocol ID or name
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum WorkersCommand {
    /// List agent workers
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get worker details
    Get {
        /// Worker ID or name
        worker_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Get worker metrics
    Metrics {
        /// Worker ID or name
        worker_id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum WorktreesCommand {
    /// List worktrees
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get worktree details
    Get {
        task_id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum MemoryCommand {
    /// List saved memories
    List {
        #[arg(long)]
        json: bool,
    },
}

// ---- Response types ----

#[derive(Debug, Deserialize, Serialize)]
struct SessionResponse {
    id: String,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProtocolResponse {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    system_prompt: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkerResponse {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    last_heartbeat: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorktreeResponse {
    #[serde(default)]
    task_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    repository_url: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentStatusResponse {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MemoryResponse {
    id: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModelResponse {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    provider: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiagnosticSessionSummary {
    id: String,
    requested_id: String,
    #[serde(default)]
    legacy_chatroom_id: Option<String>,
    tenant_id: String,
    owner_id: String,
    #[serde(default)]
    name: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiagnosticTerminalState {
    status: String,
    #[serde(default)]
    latest_terminal_event: Option<String>,
    #[serde(default)]
    latest_event_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiagnosticEventFlags {
    has_user: bool,
    has_assistant: bool,
    has_thinking: bool,
    has_tool_call: bool,
    has_tool_result: bool,
    has_tool_job_started: bool,
    has_attempt_completion: bool,
    has_usage: bool,
    has_done: bool,
    has_error: bool,
    streaming_completed: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiagnosticAnomaly {
    code: String,
    message: String,
    #[serde(default)]
    event_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiagnosticToolJobSummary {
    id: String,
    provider: String,
    status: String,
    created_at: String,
    updated_at: String,
    #[serde(default)]
    resume_session_id: Option<String>,
    #[serde(default)]
    result_session_id: Option<String>,
    has_normalized_output: bool,
    raw_event_count: usize,
    artifact_count: usize,
    #[serde(default)]
    exit_code: Option<i32>,
    #[serde(default)]
    error_message: Option<String>,
    #[serde(default)]
    estimated_nanodollar: Option<i64>,
    #[serde(default)]
    observed_nanodollar: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiagnosticExecutionStateSummary {
    id: String,
    execution_id: String,
    status: String,
    #[serde(default)]
    pending_tool_job_id: Option<String>,
    #[serde(default)]
    pending_sub_agent_execution_id: Option<String>,
    #[serde(default)]
    last_error: Option<String>,
    retry_count: u32,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentSessionDiagnosticsResponse {
    session: DiagnosticSessionSummary,
    event_count: usize,
    event_type_breakdown: std::collections::BTreeMap<String, usize>,
    #[serde(default)]
    first_event_at: Option<String>,
    #[serde(default)]
    last_event_at: Option<String>,
    terminal_state: DiagnosticTerminalState,
    flags: DiagnosticEventFlags,
    anomalies: Vec<DiagnosticAnomaly>,
    related_tool_jobs: Vec<DiagnosticToolJobSummary>,
    execution_states: Vec<DiagnosticExecutionStateSummary>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentSessionEventsResponse {
    session: DiagnosticSessionSummary,
    events: Vec<AgentSessionEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentSessionEvent {
    id: String,
    event_type: String,
    payload_json: serde_json::Value,
    created_at: String,
}

// ---- Handlers ----

async fn run_sessions_list(api: &ApiClient, json: bool) -> Result<()> {
    let sessions: Vec<SessionResponse> = api.get("/v1/llms/sessions").await?;
    if json {
        return print_json(&sessions);
    }
    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<28}  {:<12}  CREATED AT",
        "ID", "AGENT ID", "STATUS"
    );
    println!("{:-<28}  {:-<28}  {:-<12}  {:-<19}", "", "", "", "");
    for s in &sessions {
        println!(
            "{:<28}  {:<28}  {:<12}  {}",
            s.id,
            s.agent_id.as_deref().unwrap_or("-"),
            s.status.as_deref().unwrap_or("-"),
            s.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_sessions_diagnose(
    api: &ApiClient,
    session_id: &str,
    json: bool,
    fail_on: Option<FailOn>,
) -> Result<()> {
    let diagnostics: AgentSessionDiagnosticsResponse = api
        .get(&format!("/v1/llms/sessions/{session_id}/agent/diagnostics"))
        .await?;
    let should_fail = match fail_on {
        Some(FailOn::Anomaly) => !diagnostics.anomalies.is_empty(),
        Some(FailOn::Error) => diagnostics.terminal_state.status == "error",
        Some(FailOn::Incomplete) => {
            matches!(
                diagnostics.terminal_state.status.as_str(),
                "error" | "incomplete" | "running" | "empty"
            )
        }
        None => false,
    };

    if json {
        print_json(&diagnostics)?;
    } else {
        print_session_diagnostics(&diagnostics);
    }

    if should_fail {
        anyhow::bail!(
            "session diagnostics matched --fail-on {:?}: status={}, anomalies={}",
            fail_on.unwrap(),
            diagnostics.terminal_state.status,
            diagnostics.anomalies.len()
        );
    }
    Ok(())
}

async fn run_sessions_events(
    api: &ApiClient,
    session_id: &str,
    limit: Option<usize>,
    json: bool,
) -> Result<()> {
    let path = format!("/v1/llms/sessions/{session_id}/agent/events");
    let response: AgentSessionEventsResponse = match limit {
        Some(limit) => {
            let limit_str = limit.to_string();
            api.get_query(&path, &[("limit", limit_str.as_str())])
                .await?
        }
        None => api.get(&path).await?,
    };

    if json {
        return print_json(&response);
    }
    println!("Session: {}", response.session.id);
    println!("Events:  {}", response.events.len());
    for event in &response.events {
        println!(
            "{}  {:<20}  {}",
            event.created_at,
            event.event_type,
            serde_json::to_string(&event.payload_json)?
        );
    }
    Ok(())
}

fn print_session_diagnostics(d: &AgentSessionDiagnosticsResponse) {
    println!("Session:       {}", d.session.id);
    if d.session.requested_id != d.session.id {
        println!("Requested ID:  {}", d.session.requested_id);
    }
    if let Some(legacy) = &d.session.legacy_chatroom_id {
        println!("Legacy ID:     {legacy}");
    }
    println!(
        "Name:          {}",
        d.session.name.as_deref().unwrap_or("-")
    );
    println!("Tenant:        {}", d.session.tenant_id);
    println!("Owner:         {}", d.session.owner_id);
    println!("Status:        {}", d.terminal_state.status);
    println!(
        "Terminal:      {}",
        d.terminal_state
            .latest_terminal_event
            .as_deref()
            .unwrap_or("-")
    );
    println!("Events:        {}", d.event_count);
    println!(
        "First/Last:    {} / {}",
        d.first_event_at.as_deref().unwrap_or("-"),
        d.last_event_at.as_deref().unwrap_or("-")
    );
    println!(
        "SSE done:      {}",
        if d.flags.streaming_completed {
            "yes"
        } else {
            "no"
        }
    );

    if !d.event_type_breakdown.is_empty() {
        println!("Event types:");
        for (event_type, count) in &d.event_type_breakdown {
            println!("  {:<20} {}", event_type, count);
        }
    }

    if !d.related_tool_jobs.is_empty() {
        println!("Tool Jobs:");
        for job in &d.related_tool_jobs {
            println!(
                "  {}  {:<12} {:<10} normalized={} raw_events={} exit={}",
                job.id,
                job.provider,
                job.status,
                job.has_normalized_output,
                job.raw_event_count,
                job.exit_code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "-".to_string())
            );
            if let Some(error) = &job.error_message {
                println!("    error: {}", truncate(error, 160));
            }
        }
    }

    if !d.execution_states.is_empty() {
        println!("Execution States:");
        for state in &d.execution_states {
            println!(
                "  {}  {:<18} execution={} pending_job={}",
                state.id,
                state.status,
                state.execution_id,
                state.pending_tool_job_id.as_deref().unwrap_or("-")
            );
            if let Some(error) = &state.last_error {
                println!("    error: {}", truncate(error, 160));
            }
        }
    }

    if !d.anomalies.is_empty() {
        println!("Anomalies:");
        for anomaly in &d.anomalies {
            println!(
                "  {}: {}{}",
                anomaly.code,
                anomaly.message,
                anomaly
                    .event_id
                    .as_ref()
                    .map(|id| format!(" (event_id={id})"))
                    .unwrap_or_default()
            );
        }
    }
}

async fn run_protocols_list(api: &ApiClient, json: bool) -> Result<()> {
    let protocols: Vec<ProtocolResponse> = api.get("/v1/llms/agent-protocols").await?;
    if json {
        return print_json(&protocols);
    }
    if protocols.is_empty() {
        println!("No agent protocols found.");
        return Ok(());
    }
    println!("{:<28}  {:<24}  {:<16}  DESCRIPTION", "ID", "NAME", "MODEL");
    println!("{:-<28}  {:-<24}  {:-<16}  {:-<40}", "", "", "", "");
    for p in &protocols {
        println!(
            "{:<28}  {:<24}  {:<16}  {}",
            p.id,
            truncate(p.name.as_deref().unwrap_or("-"), 24),
            p.model.as_deref().unwrap_or("-"),
            truncate(p.description.as_deref().unwrap_or("-"), 40),
        );
    }
    Ok(())
}

async fn run_protocols_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let p: ProtocolResponse = api.get(&format!("/v1/llms/agent-protocols/{id}")).await?;
    if json {
        return print_json(&p);
    }
    println!("ID:          {}", p.id);
    println!("Name:        {}", p.name.as_deref().unwrap_or("-"));
    println!("Description: {}", p.description.as_deref().unwrap_or("-"));
    println!("Model:       {}", p.model.as_deref().unwrap_or("-"));
    if let Some(prompt) = &p.system_prompt {
        println!("System prompt:");
        println!("  {}", prompt.replace('\n', "\n  "));
    }
    println!("Created:     {}", p.created_at.as_deref().unwrap_or("-"));
    Ok(())
}

async fn run_workers_list(api: &ApiClient, json: bool) -> Result<()> {
    let workers: Vec<WorkerResponse> = api.get("/v1/agent/workers").await?;
    if json {
        return print_json(&workers);
    }
    if workers.is_empty() {
        println!("No workers found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<20}  {:<12}  {:<20}  CREATED AT",
        "ID", "NAME", "STATUS", "LAST HEARTBEAT"
    );
    println!(
        "{:-<28}  {:-<20}  {:-<12}  {:-<20}  {:-<19}",
        "", "", "", "", ""
    );
    for w in &workers {
        println!(
            "{:<28}  {:<20}  {:<12}  {:<20}  {}",
            w.id,
            truncate(w.name.as_deref().unwrap_or("-"), 20),
            w.status.as_deref().unwrap_or("-"),
            w.last_heartbeat.as_deref().unwrap_or("-"),
            w.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_workers_get(api: &ApiClient, worker_id: &str, json: bool) -> Result<()> {
    let w: WorkerResponse = api.get(&format!("/v1/agent/workers/{worker_id}")).await?;
    if json {
        return print_json(&w);
    }
    println!("ID:             {}", w.id);
    println!("Name:           {}", w.name.as_deref().unwrap_or("-"));
    println!("Status:         {}", w.status.as_deref().unwrap_or("-"));
    println!(
        "Last heartbeat: {}",
        w.last_heartbeat.as_deref().unwrap_or("-")
    );
    println!("Created:        {}", w.created_at.as_deref().unwrap_or("-"));
    Ok(())
}

async fn run_workers_metrics(api: &ApiClient, worker_id: &str, json: bool) -> Result<()> {
    let metrics: serde_json::Value = api
        .get(&format!("/v1/agent/workers/{worker_id}/metrics"))
        .await?;
    if json {
        return print_json(&metrics);
    }
    println!("{}", serde_json::to_string_pretty(&metrics)?);
    Ok(())
}

async fn run_worktrees_list(api: &ApiClient, json: bool) -> Result<()> {
    let worktrees: Vec<WorktreeResponse> = api.get("/v1/agent/worktrees").await?;
    if json {
        return print_json(&worktrees);
    }
    if worktrees.is_empty() {
        println!("No worktrees found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<12}  {:<24}  REPOSITORY",
        "TASK ID", "STATUS", "BRANCH"
    );
    println!("{:-<28}  {:-<12}  {:-<24}  {:-<40}", "", "", "", "");
    for w in &worktrees {
        println!(
            "{:<28}  {:<12}  {:<24}  {}",
            w.task_id.as_deref().unwrap_or("-"),
            w.status.as_deref().unwrap_or("-"),
            truncate(w.branch.as_deref().unwrap_or("-"), 24),
            truncate(w.repository_url.as_deref().unwrap_or("-"), 40),
        );
    }
    Ok(())
}

async fn run_worktrees_get(api: &ApiClient, task_id: &str, json: bool) -> Result<()> {
    let w: WorktreeResponse = api.get(&format!("/v1/agent/worktrees/{task_id}")).await?;
    if json {
        return print_json(&w);
    }
    println!("Task ID:    {}", w.task_id.as_deref().unwrap_or("-"));
    println!("Status:     {}", w.status.as_deref().unwrap_or("-"));
    println!("Branch:     {}", w.branch.as_deref().unwrap_or("-"));
    println!("Repository: {}", w.repository_url.as_deref().unwrap_or("-"));
    println!("Created:    {}", w.created_at.as_deref().unwrap_or("-"));
    Ok(())
}

async fn run_memory_list(api: &ApiClient, json: bool) -> Result<()> {
    let memories: Vec<MemoryResponse> = api.get("/v1/agent/memory").await?;
    if json {
        return print_json(&memories);
    }
    if memories.is_empty() {
        println!("No saved memories.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<12}  {:<50}  CREATED AT",
        "ID", "STATUS", "CONTENT"
    );
    println!("{:-<28}  {:-<12}  {:-<50}  {:-<19}", "", "", "", "");
    for m in &memories {
        println!(
            "{:<28}  {:<12}  {:<50}  {}",
            m.id,
            m.status.as_deref().unwrap_or("-"),
            truncate(m.content.as_deref().unwrap_or("-"), 50),
            m.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_agent_status(
    api: &ApiClient,
    agent_id: &str,
    session_id: Option<&str>,
    json: bool,
) -> Result<()> {
    let path = match session_id {
        Some(sid) => {
            format!("/v1/llms/agents/{agent_id}/sessions/{sid}/status")
        }
        None => format!("/v1/llms/agents/{agent_id}/status"),
    };
    let status: AgentStatusResponse = api.get(&path).await?;
    if json {
        return print_json(&status);
    }
    println!("Status:     {}", status.status.as_deref().unwrap_or("-"));
    println!("Agent ID:   {}", status.agent_id.as_deref().unwrap_or("-"));
    if let Some(sid) = &status.session_id {
        println!("Session ID: {sid}");
    }
    Ok(())
}

async fn run_agent_messages(
    api: &ApiClient,
    agent_id: &str,
    session_id: Option<&str>,
    json: bool,
) -> Result<()> {
    let path = match session_id {
        Some(sid) => {
            format!("/v1/llms/agents/{agent_id}/sessions/{sid}/messages")
        }
        None => format!("/v1/llms/agents/{agent_id}/messages"),
    };
    let messages: Vec<serde_json::Value> = api.get(&path).await?;
    if json {
        return print_json(&messages);
    }
    for msg in &messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("?");
        let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
        println!("[{role}] {content}");
    }
    Ok(())
}

async fn run_models_list(api: &ApiClient, json: bool) -> Result<()> {
    let models: Vec<ModelResponse> = api.get("/v1/llms/models").await?;
    if json {
        return print_json(&models);
    }
    if models.is_empty() {
        println!("No models available.");
        return Ok(());
    }
    println!("{:<32}  {:<24}  PROVIDER", "ID", "NAME");
    println!("{:-<32}  {:-<24}  {:-<16}", "", "", "");
    for m in &models {
        println!(
            "{:<32}  {:<24}  {}",
            m.id.as_deref().unwrap_or("-"),
            m.name.as_deref().unwrap_or("-"),
            m.provider.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

// ---- Entry point ----

pub async fn run(args: &AgentArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        AgentCommand::Sessions { command } => match command {
            SessionsCommand::List { json } => run_sessions_list(&api, *json).await,
            SessionsCommand::Diagnose {
                session_id,
                json,
                fail_on,
            } => run_sessions_diagnose(&api, session_id, *json, *fail_on).await,
            SessionsCommand::Events {
                session_id,
                limit,
                json,
            } => run_sessions_events(&api, session_id, *limit, *json).await,
        },
        AgentCommand::Protocols { command } => match command {
            ProtocolsCommand::List { json } => run_protocols_list(&api, *json).await,
            ProtocolsCommand::Get { id, json } => {
                let resolved = resolve::resolve_protocol_id(&api, id).await?;
                run_protocols_get(&api, &resolved, *json).await
            }
        },
        AgentCommand::Workers { command } => match command {
            WorkersCommand::List { json } => run_workers_list(&api, *json).await,
            WorkersCommand::Get { worker_id, json } => {
                let id = resolve::resolve_worker_id(&api, worker_id).await?;
                run_workers_get(&api, &id, *json).await
            }
            WorkersCommand::Metrics { worker_id, json } => {
                let id = resolve::resolve_worker_id(&api, worker_id).await?;
                run_workers_metrics(&api, &id, *json).await
            }
        },
        AgentCommand::Worktrees { command } => match command {
            WorktreesCommand::List { json } => run_worktrees_list(&api, *json).await,
            WorktreesCommand::Get { task_id, json } => {
                run_worktrees_get(&api, task_id, *json).await
            }
        },
        AgentCommand::Memory { command } => match command {
            MemoryCommand::List { json } => run_memory_list(&api, *json).await,
        },
        AgentCommand::Status {
            agent_id,
            session_id,
            json,
        } => run_agent_status(&api, agent_id, session_id.as_deref(), *json).await,
        AgentCommand::Messages {
            agent_id,
            session_id,
            json,
        } => run_agent_messages(&api, agent_id, session_id.as_deref(), *json).await,
        AgentCommand::Models { json } => run_models_list(&api, *json).await,
    }
}
