mod response_contract_support;

use response_contract_support::{run_cases, Case};

#[test]
fn agent_responses_use_exact_api_fields() {
    run_cases(&[
        Case {
            label: "agent session",
            args: &["agent", "sessions", "list", "--json"],
            path: "/v1/llms/sessions",
            valid_body: r#"{"sessions":[{"id":"as_123456789012","name":"session-one","created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            invalid_body: r#"{"sessions":[{"id":"as_123456789012","agent_id":"agent-one","status":"running","created_at":"2026-08-01T00:00:00Z"}]}"#,
            missing_field: "updated_at",
            forbidden_output: None,
        },
        Case {
            label: "agent protocol",
            args: &["agent", "protocols", "list", "--json"],
            path: "/v1/llms/agent-protocols",
            valid_body: r#"{"items":[{"id":"ap_123456789012","tenant_id":"tn_test1234567890","title":"Protocol One","protocol_name":"protocol-one","description":null,"markdown":"Protocol body","created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            invalid_body: r#"{"items":[{"id":"ap_123456789012","tenant_id":"tn_test1234567890","title":"Protocol One","name":"protocol-one","description":null,"markdown":"Protocol body","created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            missing_field: "protocol_name",
            forbidden_output: None,
        },
        Case {
            label: "agent worker",
            args: &["agent", "workers", "list", "--json"],
            path: "/v1/agent/workers",
            valid_body: r#"{"workers":[{"id":"wrk_123456789012","name":"worker-one","status":"online","last_heartbeat_at":"2026-08-02T00:00:00Z","created_at":"2026-08-01T00:00:00Z"}]}"#,
            invalid_body: r#"{"workers":[{"id":"wrk_123456789012","name":"worker-one","worker_status":"online","last_heartbeat":"2026-08-02T00:00:00Z","created_at":"2026-08-01T00:00:00Z"}]}"#,
            missing_field: "status",
            forbidden_output: None,
        },
        Case {
            label: "agent worktree",
            args: &["agent", "worktrees", "list", "--json"],
            path: "/v1/agent/worktrees",
            valid_body: r#"{"worktrees":[{"path":"/tmp/worktree","branch_name":"feat/test","task_id":"task-1","status":"ready"}],"total":1}"#,
            invalid_body: r#"{"worktrees":[{"repository_url":"repo","branch":"feat/test","task_id":"task-1","status":"ready"}],"total":1}"#,
            missing_field: "path",
            forbidden_output: None,
        },
        Case {
            label: "agent status",
            args: &["agent", "status", "agent-one", "--json"],
            path: "/v1/llms/agents/agent-one/status",
            valid_body: r#"{"is_running":true,"progress":40,"state":"running"}"#,
            invalid_body: r#"{"status":"running","agent_id":"agent-one","session_id":null}"#,
            missing_field: "is_running",
            forbidden_output: None,
        },
        Case {
            label: "saved memory",
            args: &["agent", "memory", "list", "--json"],
            path: "/v1/agent/memory",
            valid_body: r#"{"memories":[{"id":"mem_123456789012","clause":"Remember this","raw_facts":["fact"],"status":"active","source":"user","created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            invalid_body: r#"{"memories":[{"id":"mem_123456789012","content":"Remember this","status":"active","created_at":"2026-08-01T00:00:00Z"}]}"#,
            missing_field: "clause",
            forbidden_output: None,
        },
    ]);
}
