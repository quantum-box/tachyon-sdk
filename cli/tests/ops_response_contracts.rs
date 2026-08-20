mod response_contract_support;

use response_contract_support::{run_cases, Case};

#[test]
fn ops_responses_use_exact_api_fields() {
    run_cases(&[
        Case {
            label: "scenario report envelope",
            args: &["ops", "reports", "list", "--json"],
            path: "/v1/ops/scenario-reports",
            valid_body: r#"{"test_runs":[],"total":0}"#,
            invalid_body: r#"{"test_runs":[],"count":0}"#,
            missing_field: "total",
            forbidden_output: None,
        },
        Case {
            label: "scenario report",
            args: &["ops", "reports", "list", "--json"],
            path: "/v1/ops/scenario-reports",
            valid_body: r#"{"test_runs":[{"id":"tr_123456789012","status":"passed","total_scenarios":3,"passed_scenarios":3,"failed_scenarios":0,"total_duration_ms":1200,"ci_repository":"quantum-box/example","ci_branch":"main","ci_commit_sha":"abc123","ci_pull_request_number":42,"created_at":"2026-08-01T00:00:00Z"}],"total":1}"#,
            invalid_body: r#"{"test_runs":[{"run_id":"tr_123456789012","status":"passed","total_scenarios":3,"passed_scenarios":3,"failed_scenarios":0,"total_duration_ms":1200,"ci_repository":"quantum-box/example","ci_branch":"main","ci_commit_sha":"abc123","ci_pull_request_number":42,"created_at":"2026-08-01T00:00:00Z"}],"total":1}"#,
            missing_field: "id",
            forbidden_output: None,
        },
        Case {
            label: "ops deployment list envelope",
            args: &["ops", "deployments", "list", "--json"],
            path: "/v1/ops/deployments",
            valid_body: r#"{"deployments":[],"total":0}"#,
            invalid_body: r#"{"deployments":[],"count":0}"#,
            missing_field: "total",
            forbidden_output: None,
        },
        Case {
            label: "ops deployment list",
            args: &["ops", "deployments", "list", "--json"],
            path: "/v1/ops/deployments",
            valid_body: r#"{"deployments":[{"id":"dep_123456789012","service_name":"api","version":"v1","environment":"production","status":"succeeded","ci_branch":"main","ci_commit_sha":"abc123","ci_run_url":"https://ci.invalid/run/1","started_at":"2026-08-01T00:00:00Z","completed_at":"2026-08-01T00:00:01Z","duration_ms":1000}],"total":1}"#,
            invalid_body: r#"{"deployments":[{"id":"dep_123456789012","service":"api","version":"v1","environment":"production","status":"succeeded","ci_branch":"main","ci_commit_sha":"abc123","ci_run_url":"https://ci.invalid/run/1","started_at":"2026-08-01T00:00:00Z","completed_at":"2026-08-01T00:00:01Z","duration_ms":1000}],"total":1}"#,
            missing_field: "service_name",
            forbidden_output: None,
        },
        Case {
            label: "ops deployment detail",
            args: &["ops", "deployments", "get", "dep_123456789012", "--json"],
            path: "/v1/ops/deployments/dep_123456789012",
            valid_body: r#"{"id":"dep_123456789012","operator_id":"op_123456789012","service_name":"api","version":"v1","environment":"production","status":"succeeded","ci_provider":"github","ci_run_url":"https://ci.invalid/run/1","ci_repository":"quantum-box/example","ci_branch":"main","ci_commit_sha":"abc123","error_message":null,"started_at":"2026-08-01T00:00:00Z","completed_at":"2026-08-01T00:00:01Z","duration_ms":1000,"created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-01T00:00:01Z"}"#,
            invalid_body: r#"{"id":"dep_123456789012","operator_id":"op_123456789012","service":"api","version":"v1","environment":"production","status":"succeeded","ci_provider":"github","ci_run_url":"https://ci.invalid/run/1","ci_repository":"quantum-box/example","ci_branch":"main","ci_commit_sha":"abc123","error_message":null,"started_at":"2026-08-01T00:00:00Z","completed_at":"2026-08-01T00:00:01Z","duration_ms":1000,"created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-01T00:00:01Z"}"#,
            missing_field: "service_name",
            forbidden_output: None,
        },
        Case {
            label: "coding job",
            args: &["ops", "coding-jobs", "list", "--json"],
            path: "/v1/agent/coding-jobs",
            valid_body: r#"{"jobs":[{"coding_job_id":"cj_123456789012","provider":"codex","status":"succeeded","prompt":"Fix it","context_paths":[],"output_profile":null,"environment":{},"metadata":{},"executor":{"operator_id":"op_123456789012"},"use_worktree":false,"auto_merge":false,"created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            invalid_body: r#"{"jobs":[{"id":"cj_123456789012","provider":"codex","status":"succeeded","prompt":"Fix it","context_paths":[],"output_profile":null,"environment":{},"metadata":{},"executor":{"operator_id":"op_123456789012"},"tool_name":"codex","use_worktree":false,"auto_merge":false,"created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            missing_field: "coding_job_id",
            forbidden_output: None,
        },
    ]);
}
