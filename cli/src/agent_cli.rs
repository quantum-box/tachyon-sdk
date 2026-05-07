use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    #[command(alias = "session")]
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
    /// Show a human-readable session diagnostic summary
    Inspect {
        /// Session ID (`as_...`) or legacy chatroom ID (`ch_...`)
        session_id: String,
        /// Fail with a non-zero exit when the selected condition is present
        #[arg(long, value_parser = ["anomaly", "error", "incomplete"])]
        fail_on: Option<String>,
    },
    /// Print raw session event timeline
    Events {
        /// Session ID (`as_...`) or legacy chatroom ID (`ch_...`)
        session_id: String,
        /// Maximum number of events to print
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        json: bool,
    },
    /// Print machine-readable diagnostics
    Diagnose {
        /// Session ID (`as_...`) or legacy chatroom ID (`ch_...`)
        session_id: String,
        /// Deprecated; use `events --limit` to retrieve the raw timeline
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        json: bool,
        /// Fail with a non-zero exit when the selected condition is present
        #[arg(long, value_parser = ["anomaly", "error", "incomplete"])]
        fail_on: Option<String>,
    },
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
struct SessionsListResponse {
    sessions: Vec<SessionResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentDiagnosticsSessionSummary {
    id: String,
    requested_id: String,
    #[serde(default)]
    legacy_chatroom_id: Option<String>,
    tenant_id: String,
    owner_id: String,
    #[serde(default)]
    name: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentDiagnosticsEvent {
    id: String,
    event_type: String,
    payload_json: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentDiagnosticsExecutionState {
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
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentDiagnosticsToolJob {
    id: String,
    provider: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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

#[derive(Debug, Deserialize, Serialize, Default)]
struct AgentDiagnosticsFlags {
    #[serde(default)]
    has_user: bool,
    #[serde(default)]
    has_assistant: bool,
    #[serde(default)]
    has_thinking: bool,
    #[serde(default)]
    has_tool_call: bool,
    #[serde(default)]
    has_tool_result: bool,
    #[serde(default)]
    has_tool_job_started: bool,
    #[serde(default)]
    has_attempt_completion: bool,
    #[serde(default)]
    has_usage: bool,
    #[serde(default)]
    has_done: bool,
    #[serde(default)]
    has_error: bool,
    #[serde(default)]
    streaming_completed: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentDiagnosticsTerminalState {
    status: String,
    #[serde(default)]
    latest_terminal_event: Option<String>,
    #[serde(default)]
    latest_event_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentDiagnosticsAnomaly {
    code: String,
    message: String,
    #[serde(default)]
    event_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentDiagnosticsResponse {
    session: AgentDiagnosticsSessionSummary,
    event_count: usize,
    event_type_breakdown: BTreeMap<String, usize>,
    first_event_at: Option<DateTime<Utc>>,
    last_event_at: Option<DateTime<Utc>>,
    terminal_state: AgentDiagnosticsTerminalState,
    #[serde(default)]
    flags: AgentDiagnosticsFlags,
    anomalies: Vec<AgentDiagnosticsAnomaly>,
    related_tool_jobs: Vec<AgentDiagnosticsToolJob>,
    execution_states: Vec<AgentDiagnosticsExecutionState>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentEventsResponse {
    session: AgentDiagnosticsSessionSummary,
    events: Vec<AgentDiagnosticsEvent>,
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

// ---- Handlers ----

async fn run_sessions_list(api: &ApiClient, json: bool) -> Result<()> {
    let response: SessionsListResponse = api.get("/v1/llms/sessions").await?;
    let sessions = response.sessions;
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

async fn fetch_session_diagnostics(
    api: &ApiClient,
    session_id: &str,
) -> Result<AgentDiagnosticsResponse> {
    let path = format!("/v1/llms/sessions/{session_id}/agent/diagnostics");
    api.get(&path).await
}

async fn fetch_session_events(
    api: &ApiClient,
    session_id: &str,
    limit: Option<usize>,
) -> Result<AgentEventsResponse> {
    let path = match limit {
        Some(limit) => format!("/v1/llms/sessions/{session_id}/agent/events?limit={limit}"),
        None => format!("/v1/llms/sessions/{session_id}/agent/events"),
    };
    api.get(&path).await
}

fn check_fail_on(diagnostics: &AgentDiagnosticsResponse, fail_on: Option<&str>) -> Result<()> {
    match fail_on {
        Some("anomaly") if !diagnostics.anomalies.is_empty() => Err(anyhow!(
            "session diagnostics reported {} anomalies",
            diagnostics.anomalies.len()
        )),
        Some("error")
            if diagnostics.terminal_state.status == "error" || diagnostics.flags.has_error =>
        {
            Err(anyhow!(
                "session diagnostics reported an error terminal state"
            ))
        }
        Some("incomplete") if diagnostics.terminal_state.status != "completed" => Err(anyhow!(
            "session diagnostics status is {}",
            diagnostics.terminal_state.status
        )),
        _ => Ok(()),
    }
}

fn enabled_flags(flags: &AgentDiagnosticsFlags) -> Vec<&'static str> {
    let mut values = Vec::new();
    if flags.has_user {
        values.push("user");
    }
    if flags.has_assistant {
        values.push("assistant");
    }
    if flags.has_thinking {
        values.push("thinking");
    }
    if flags.has_tool_call {
        values.push("tool_call");
    }
    if flags.has_tool_result {
        values.push("tool_result");
    }
    if flags.has_tool_job_started {
        values.push("tool_job_started");
    }
    if flags.has_attempt_completion {
        values.push("attempt_completion");
    }
    if flags.has_usage {
        values.push("usage");
    }
    if flags.has_done {
        values.push("done");
    }
    if flags.has_error {
        values.push("error");
    }
    if flags.streaming_completed {
        values.push("streaming_completed");
    }
    values
}

async fn run_session_inspect(
    api: &ApiClient,
    session_id: &str,
    fail_on: Option<&str>,
) -> Result<()> {
    let diagnostics = fetch_session_diagnostics(api, session_id).await?;
    println!("Session:          {}", diagnostics.session.id);
    if diagnostics.session.requested_id != diagnostics.session.id {
        println!("Requested ID:     {}", diagnostics.session.requested_id);
    }
    if let Some(chatroom_id) = &diagnostics.session.legacy_chatroom_id {
        println!("Chatroom ID:      {chatroom_id}");
    }
    println!("Terminal state:   {}", diagnostics.terminal_state.status);
    if let Some(event_type) = &diagnostics.terminal_state.latest_event_type {
        println!("Latest event:     {event_type}");
    }
    println!("Events:           {}", diagnostics.event_count);
    println!(
        "First event:      {}",
        diagnostics
            .first_event_at
            .map(|value| value.to_rfc3339())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Last event:       {}",
        diagnostics
            .last_event_at
            .map(|value| value.to_rfc3339())
            .unwrap_or_else(|| "-".to_string())
    );

    let flags = enabled_flags(&diagnostics.flags);
    if !flags.is_empty() {
        println!();
        println!("Flags:            {}", flags.join(", "));
    }

    if !diagnostics.execution_states.is_empty() {
        println!();
        println!("Execution States:");
        for state in &diagnostics.execution_states {
            println!(
                "  {}  {:<12} retries={}",
                state.execution_id, state.status, state.retry_count
            );
            if let Some(job_id) = &state.pending_tool_job_id {
                println!("    pending job: {job_id}");
            }
            if let Some(execution_id) = &state.pending_sub_agent_execution_id {
                println!("    pending sub-agent execution: {execution_id}");
            }
            if let Some(error) = &state.last_error {
                println!("    error: {error}");
            }
        }
    }

    println!();
    println!("Event breakdown:");
    for (event_type, count) in &diagnostics.event_type_breakdown {
        println!("  {event_type:<24} {count}");
    }

    if !diagnostics.related_tool_jobs.is_empty() {
        println!();
        println!("Related Tool Jobs:");
        for job in &diagnostics.related_tool_jobs {
            println!(
                "  {}  {:<12} {:<12} raw_events={} artifacts={}",
                job.id, job.provider, job.status, job.raw_event_count, job.artifact_count
            );
            if let Some(session_id) = &job.result_session_id {
                println!("    result session: {session_id}");
            }
            if let Some(session_id) = &job.resume_session_id {
                println!("    resume session: {session_id}");
            }
            if let Some(exit_code) = job.exit_code {
                println!("    exit code: {exit_code}");
            }
            if let Some(error) = &job.error_message {
                println!("    error: {error}");
            }
        }
    }

    if !diagnostics.anomalies.is_empty() {
        println!();
        println!("Anomalies:");
        for anomaly in &diagnostics.anomalies {
            match &anomaly.event_id {
                Some(event_id) => {
                    println!("  - [{}] {} ({event_id})", anomaly.code, anomaly.message)
                }
                None => println!("  - [{}] {}", anomaly.code, anomaly.message),
            }
        }
    }

    check_fail_on(&diagnostics, fail_on)
}

async fn run_session_events(
    api: &ApiClient,
    session_id: &str,
    limit: Option<usize>,
    json: bool,
) -> Result<()> {
    let response = fetch_session_events(api, session_id, limit).await?;
    if json {
        return print_json(&response);
    }
    if response.events.is_empty() {
        println!("No events found.");
        return Ok(());
    }
    println!("Session: {}", response.session.id);
    println!();
    println!("{:<28}  {:<22}  TYPE", "CREATED AT", "EVENT ID");
    println!("{:-<28}  {:-<22}  {:-<24}", "", "", "");
    for event in &response.events {
        println!(
            "{:<28}  {:<22}  {}",
            event.created_at.to_rfc3339(),
            event.id,
            event.event_type
        );
    }
    Ok(())
}

async fn run_session_diagnose(
    api: &ApiClient,
    session_id: &str,
    limit: Option<usize>,
    json: bool,
    fail_on: Option<&str>,
) -> Result<()> {
    let diagnostics = fetch_session_diagnostics(api, session_id).await?;
    if limit.is_some() && !json {
        eprintln!("warning: --limit is ignored for diagnose; use `events --limit` for raw events");
    }
    if json {
        print_json(&diagnostics)?;
    } else {
        println!(
            "{}: {} ({} events, {} anomalies)",
            diagnostics.session.id,
            diagnostics.terminal_state.status,
            diagnostics.event_count,
            diagnostics.anomalies.len()
        );
    }
    check_fail_on(&diagnostics, fail_on)
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
            SessionsCommand::Inspect {
                session_id,
                fail_on,
            } => run_session_inspect(&api, session_id, fail_on.as_deref()).await,
            SessionsCommand::Events {
                session_id,
                limit,
                json,
            } => run_session_events(&api, session_id, *limit, *json).await,
            SessionsCommand::Diagnose {
                session_id,
                limit,
                json,
                fail_on,
            } => run_session_diagnose(&api, session_id, *limit, *json, fail_on.as_deref()).await,
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
