mod response_contract_support;

use response_contract_support::{run_cases, Case};

#[test]
fn ops_responses_use_exact_api_fields() {
    run_cases(&[
        Case {
            label: "scenario report",
            args: &["ops", "reports", "list", "--json"],
            path: "/v1/ops/scenario-reports",
            valid_body: r#"{"test_runs":[{"id":"tr_123456789012","status":"passed","total_scenarios":3,"passed_scenarios":3,"failed_scenarios":0,"created_at":"2026-08-01T00:00:00Z"}],"total":1}"#,
            invalid_body: r#"{"test_runs":[{"run_id":"tr_123456789012","status":"passed","total_scenarios":3,"passed_scenarios":3,"failed_scenarios":0,"created_at":"2026-08-01T00:00:00Z"}],"total":1}"#,
            missing_field: "id",
            forbidden_output: None,
        },
        Case {
            label: "ops deployment list",
            args: &["ops", "deployments", "list", "--json"],
            path: "/v1/ops/deployments",
            valid_body: r#"{"deployments":[{"id":"dep_123456789012","service_name":"api","version":"v1","environment":"production","status":"succeeded","started_at":"2026-08-01T00:00:00Z"}],"total":1}"#,
            invalid_body: r#"{"deployments":[{"id":"dep_123456789012","service":"api","version":"v1","environment":"production","status":"succeeded","created_at":"2026-08-01T00:00:00Z"}],"total":1}"#,
            missing_field: "service_name",
            forbidden_output: None,
        },
        Case {
            label: "ops deployment detail",
            args: &["ops", "deployments", "get", "dep_123456789012", "--json"],
            path: "/v1/ops/deployments/dep_123456789012",
            valid_body: r#"{"id":"dep_123456789012","service_name":"api","version":"v1","environment":"production","status":"succeeded","created_at":"2026-08-01T00:00:00Z"}"#,
            invalid_body: r#"{"id":"dep_123456789012","service":"api","version":"v1","environment":"production","status":"succeeded","created_at":"2026-08-01T00:00:00Z"}"#,
            missing_field: "service_name",
            forbidden_output: None,
        },
        Case {
            label: "coding job",
            args: &["ops", "coding-jobs", "list", "--json"],
            path: "/v1/agent/coding-jobs",
            valid_body: r#"{"jobs":[{"coding_job_id":"cj_123456789012","provider":"codex","status":"succeeded","prompt":"Fix it","context_paths":[],"created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            invalid_body: r#"{"jobs":[{"id":"cj_123456789012","provider":"codex","status":"succeeded","prompt":"Fix it","context_paths":[],"tool_name":"codex","created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            missing_field: "coding_job_id",
            forbidden_output: None,
        },
    ]);
}
