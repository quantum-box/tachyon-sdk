mod response_contract_support;

use response_contract_support::{run_cases, Case};

#[test]
fn iac_and_domain_responses_use_exact_api_fields() {
    run_cases(&[
        Case {
            label: "integration list",
            args: &["iac", "integrations", "list", "--json"],
            path: "/v1/integrations",
            valid_body: r#"{"integrations":[{"id":"int_123456789012","name":"GitHub","description":"Source control","category":"code_management","provider":"github","icon_url":null,"is_enabled":true,"is_featured":true,"requires_oauth":true,"requires_setup":false}]}"#,
            invalid_body: r#"{"integrations":[{"id":"int_123456789012","name":"GitHub","description":"Source control","category":"code_management","provider":"github","icon_url":null,"status":"enabled","is_featured":true,"requires_oauth":true,"requires_setup":false}]}"#,
            missing_field: "is_enabled",
            forbidden_output: None,
        },
        Case {
            label: "integration detail",
            args: &["iac", "integrations", "get", "int_123456789012", "--json"],
            path: "/v1/integrations/int_123456789012",
            valid_body: r#"{"id":"int_123456789012","name":"GitHub","description":"Source control","category":"code_management","provider":"github","icon_url":null,"sync_capability":"bidirectional","supported_objects":["issues"],"is_enabled":true,"is_featured":true,"requires_oauth":true,"oauth_scopes":["repo"]}"#,
            invalid_body: r#"{"id":"int_123456789012","name":"GitHub","description":"Source control","category":"code_management","provider":"github","icon_url":null,"sync_capability":"bidirectional","supported_objects":["issues"],"status":"enabled","is_featured":true,"requires_oauth":true,"oauth_scopes":["repo"]}"#,
            missing_field: "is_enabled",
            forbidden_output: None,
        },
        Case {
            label: "oauth provider envelope",
            args: &["iac", "oauth-providers", "--json"],
            path: "/v1/iac/oauth-providers?tenant_id=tn_test1234567890",
            valid_body: r#"{"providers":[]}"#,
            invalid_body: r#"{"github":null,"linear":null}"#,
            missing_field: "providers",
            forbidden_output: None,
        },
        Case {
            label: "oauth provider",
            args: &["iac", "oauth-providers", "--json"],
            path: "/v1/iac/oauth-providers?tenant_id=tn_test1234567890",
            valid_body: r#"{"providers":[{"provider":"github","client_id":"client-id","client_secret":"secret-value","redirect_uri":"https://example.invalid/callback","webhook_secret":null}]}"#,
            invalid_body: r#"{"providers":[{"provider":"github","clientId":"client-id","client_secret":"secret-value","redirect_uri":"https://example.invalid/callback"}]}"#,
            missing_field: "client_id",
            forbidden_output: Some("secret-value"),
        },
        Case {
            label: "integration connection envelope",
            args: &["iac", "connections", "list", "--json"],
            path: "/v1/integrations/connections",
            valid_body: r#"{"connections":[]}"#,
            invalid_body: r#"{"items":[]}"#,
            missing_field: "connections",
            forbidden_output: None,
        },
        Case {
            label: "integration connection",
            args: &["iac", "connections", "list", "--json"],
            path: "/v1/integrations/connections",
            valid_body: r#"{"connections":[{"id":"conn_123456789012","integration_id":"int_123456789012","provider":"github","status":"connected","external_account_id":"123","external_account_name":"quantum-box","connected_at":"2026-08-01T00:00:00Z","last_synced_at":null,"error_message":null,"metadata":{}}]}"#,
            invalid_body: r#"{"connections":[{"id":"conn_123456789012","integration_id":"int_123456789012","provider":"github","status":"connected","account_name":"quantum-box","created_at":"2026-08-01T00:00:00Z","metadata":{}}]}"#,
            missing_field: "connected_at",
            forbidden_output: None,
        },
        Case {
            label: "custom domain",
            args: &["compute", "domains", "list", "app_123456789012", "--json"],
            path: "/v1/compute/apps/app_123456789012/domains",
            valid_body: r#"{"domains":[{"id":"dom_123456789012","app_id":"app_123456789012","domain":"example.invalid","status":"active","tls_status":"active","cname_target":"customers.txcloud.app","created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            invalid_body: r#"{"domains":[{"id":"dom_123456789012","app_id":"app_123456789012","domain":"example.invalid","status":"active","verified":true,"cname_target":"customers.txcloud.app","created_at":"2026-08-01T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}]}"#,
            missing_field: "tls_status",
            forbidden_output: None,
        },
    ]);
}
