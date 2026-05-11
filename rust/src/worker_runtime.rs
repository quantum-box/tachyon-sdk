//! Worker runtime boundary for Tachyon Tool Jobs.
//!
//! This module contains the stable traits and DTOs needed by a worker
//! runtime. It intentionally avoids Tachyon server-side crates so SDK
//! users can implement external workers without depending on the
//! monorepo core packages.

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result type used by SDK worker runtime traits.
pub type WorkerRuntimeResult<T> = Result<T, WorkerRuntimeError>;

/// Lightweight runtime error for worker boundary implementations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerRuntimeError {
    /// Stable machine-readable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

impl WorkerRuntimeError {
    /// Creates a new worker runtime error.
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for WorkerRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for WorkerRuntimeError {}

/// Provider selected for a Tool Job.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum WorkerProviderKind {
    /// Codex CLI provider.
    Codex,
    /// Claude Code CLI provider.
    ClaudeCode,
    /// Cursor Agent provider.
    CursorAgent,
    /// OpenCode provider.
    OpenCode,
    /// Codex CLI running inside a Docker container.
    ContainerizedCodex,
}

impl fmt::Display for WorkerProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Codex => write!(f, "codex"),
            Self::ClaudeCode => write!(f, "claude_code"),
            Self::CursorAgent => write!(f, "cursor_agent"),
            Self::OpenCode => write!(f, "open_code"),
            Self::ContainerizedCodex => write!(f, "containerized_codex"),
        }
    }
}

/// Actor context supplied to a worker execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkerExecutorContext {
    /// Operator tenant ID.
    pub operator_id: String,
    /// Optional end-user actor ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Optional platform tenant ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_id: Option<String>,
}

/// Payload stored in a runtime queue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerJobPayload {
    /// Execution state ID that owns this job.
    pub execution_state_id: String,
    /// Tool Job ID for tracking.
    pub tool_job_id: String,
    /// Provider name as received from the API or queue backend.
    pub provider: String,
    /// Prompt or task to execute.
    pub prompt: String,
    /// Current retry count.
    pub retry_count: u32,
    /// Additional provider/runtime metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Queue entry status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkerQueuedJobStatus {
    /// Job is waiting to be processed.
    Pending,
    /// Job is currently being processed.
    Processing,
    /// Job completed successfully.
    Completed,
    /// Job failed permanently.
    Failed,
}

/// Queue entry delivered to a worker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerQueuedJob {
    /// Backend-specific queue entry ID.
    pub queue_id: String,
    /// Tool Job ID.
    pub job_id: String,
    /// Job payload.
    pub payload: WorkerJobPayload,
    /// Current queue status.
    pub status: WorkerQueuedJobStatus,
    /// Time when this entry becomes visible.
    pub visible_at: DateTime<Utc>,
    /// Number of attempts observed by the queue backend.
    pub attempt_count: u32,
    /// Entry creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Entry update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Request passed to a job executor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerJobRequest {
    /// Tool Job ID.
    pub job_id: String,
    /// Selected provider.
    pub provider: WorkerProviderKind,
    /// Prompt or task to execute.
    pub prompt: String,
    /// Optional context paths.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_paths: Vec<String>,
    /// Optional output profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_profile: Option<String>,
    /// Environment variables supplied to the execution.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, String>,
    /// Runtime metadata.
    #[serde(default)]
    pub metadata: Value,
    /// Executor actor context.
    #[serde(default)]
    pub executor: WorkerExecutorContext,
    /// Optional provider session to resume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_session_id: Option<String>,
    /// Whether the worker should execute in a temporary git worktree.
    #[serde(default)]
    pub use_worktree: bool,
    /// Whether generated changes can be merged automatically.
    #[serde(default)]
    pub auto_merge: bool,
}

/// Normalized output returned by a worker job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerNormalizedOutput {
    /// Output format identifier.
    pub format: String,
    /// Output body.
    pub body: Value,
}

/// Event captured during worker execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerJobEvent {
    /// Event type name.
    pub event_type: String,
    /// Optional event payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
}

/// Artifact created by a worker job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerJobArtifact {
    /// Artifact type name.
    pub artifact_type: String,
    /// Artifact reference such as a URL, path, or object key.
    pub reference: String,
    /// Optional artifact metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Worker job execution result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkerJobResult {
    /// Normalized provider output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_output: Option<WorkerNormalizedOutput>,
    /// Raw events captured during execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_events: Vec<WorkerJobEvent>,
    /// Artifacts created during execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<WorkerJobArtifact>,
    /// Provider process exit code if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Provider session ID if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Stream event emitted by a worker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerStreamEvent {
    /// Text delta from a provider.
    TextDelta { text: String },
    /// Status change emitted by the worker.
    StatusChange {
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Tool execution start.
    ToolStart {
        tool_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_preview: Option<String>,
    },
    /// Tool execution end.
    ToolEnd {
        tool_name: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        output_preview: Option<String>,
    },
    /// Final successful completion.
    Done {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        prompt_tokens: u64,
        completion_tokens: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },
    /// Execution error.
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
}

/// Queue abstraction used by worker runtimes.
#[async_trait]
pub trait WorkerJobQueue: Send + Sync + fmt::Debug {
    /// Enqueue a job for immediate processing.
    async fn enqueue(
        &self,
        job_id: &str,
        payload: WorkerJobPayload,
    ) -> WorkerRuntimeResult<String>;

    /// Enqueue a job after a delay.
    async fn enqueue_with_delay(
        &self,
        job_id: &str,
        payload: WorkerJobPayload,
        delay: Duration,
    ) -> WorkerRuntimeResult<String>;

    /// Dequeue the next available job.
    async fn dequeue(&self)
        -> WorkerRuntimeResult<Option<WorkerQueuedJob>>;

    /// Acknowledge successful job processing.
    async fn ack(&self, queue_id: &str) -> WorkerRuntimeResult<()>;

    /// Return a job to the queue for retry.
    async fn nack(
        &self,
        queue_id: &str,
        delay: Duration,
    ) -> WorkerRuntimeResult<()>;

    /// Return the current pending job count when the backend can provide it.
    async fn pending_count(&self) -> WorkerRuntimeResult<u64>;

    /// Fetch a queued job by backend queue ID.
    async fn get(
        &self,
        queue_id: &str,
    ) -> WorkerRuntimeResult<Option<WorkerQueuedJob>>;
}

/// Executes one worker job.
#[async_trait]
pub trait JobExecutor: Send + Sync + fmt::Debug {
    /// Execute a job request and return its normalized result.
    async fn execute(
        &self,
        request: WorkerJobRequest,
    ) -> WorkerRuntimeResult<WorkerJobResult>;
}

/// Publishes streaming events for a worker job.
#[async_trait]
pub trait WorkerEventPublisher: Send + Sync + fmt::Debug {
    /// Publish an event for a specific Tool Job.
    async fn publish(
        &self,
        job_id: &str,
        event: WorkerStreamEvent,
    ) -> WorkerRuntimeResult<()>;

    /// Close event resources for a job.
    async fn close(&self, job_id: &str) -> WorkerRuntimeResult<()>;
}

/// Runtime lifecycle handle for a worker process.
#[async_trait]
pub trait WorkerHandle: Send + Sync + fmt::Debug {
    /// Stable worker ID registered with Tachyon.
    fn worker_id(&self) -> &str;

    /// Start the worker runtime.
    async fn start(&self) -> WorkerRuntimeResult<()>;

    /// Request graceful shutdown.
    async fn shutdown(&self) -> WorkerRuntimeResult<()>;

    /// Number of currently active jobs.
    fn active_job_count(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn queued_job_roundtrip_preserves_runtime_fields() {
        let now = Utc::now();
        let job = WorkerQueuedJob {
            queue_id: "queue-1".to_string(),
            job_id: "job-1".to_string(),
            payload: WorkerJobPayload {
                execution_state_id: "exec-1".to_string(),
                tool_job_id: "job-1".to_string(),
                provider: "codex".to_string(),
                prompt: "Write tests".to_string(),
                retry_count: 2,
                metadata: Some(serde_json::json!({"use_worktree": true})),
            },
            status: WorkerQueuedJobStatus::Processing,
            visible_at: now,
            attempt_count: 3,
            created_at: now,
            updated_at: now,
        };

        let encoded = serde_json::to_string(&job).unwrap();
        let decoded: WorkerQueuedJob =
            serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, job);
    }

    #[derive(Debug, Default)]
    struct RecordingExecutor {
        prompts: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl JobExecutor for RecordingExecutor {
        async fn execute(
            &self,
            request: WorkerJobRequest,
        ) -> WorkerRuntimeResult<WorkerJobResult> {
            self.prompts.lock().unwrap().push(request.prompt);
            Ok(WorkerJobResult {
                normalized_output: Some(WorkerNormalizedOutput {
                    format: "json".to_string(),
                    body: serde_json::json!({"ok": true}),
                }),
                ..WorkerJobResult::default()
            })
        }
    }

    #[tokio::test]
    async fn job_executor_can_be_used_as_trait_object() {
        let executor = Arc::new(RecordingExecutor::default());
        let boxed: Arc<dyn JobExecutor> = executor.clone();

        let result = boxed
            .execute(WorkerJobRequest {
                job_id: "job-1".to_string(),
                provider: WorkerProviderKind::Codex,
                prompt: "Do work".to_string(),
                context_paths: vec![],
                output_profile: None,
                environment: HashMap::new(),
                metadata: Value::Null,
                executor: WorkerExecutorContext::default(),
                resume_session_id: None,
                use_worktree: false,
                auto_merge: false,
            })
            .await
            .unwrap();

        assert_eq!(
            executor.prompts.lock().unwrap().as_slice(),
            ["Do work"]
        );
        assert_eq!(
            result.normalized_output.unwrap().body,
            serde_json::json!({"ok": true})
        );
    }
}
