mod response_contract_support;

use response_contract_support::{run_cases, Case};

#[test]
fn auth_responses_use_exact_api_fields() {
    run_cases(&[
        Case {
            label: "operator",
            args: &["org", "operators", "list", "--json"],
            path: "/v1/auth/operators/by-user",
            valid_body: r#"[{"id":"tn_operator1234567890","name":"Operator","operatorName":"operator-one","platformId":"tn_platform123456789"}]"#,
            invalid_body: r#"[{"id":"tn_operator1234567890","name":"Operator","alias":"operator-one","platformId":"tn_platform123456789"}]"#,
            missing_field: "operatorName",
            forbidden_output: None,
        },
        Case {
            label: "policy action",
            args: &["org", "policies", "actions", "--json"],
            path: "/v1/auth/actions",
            valid_body: r#"{"actions":[{"id":"act_123456789012","platformId":null,"sharedWithDescendants":false,"ownerTenantId":null,"context":"finance","name":"ListInvoices","fullName":"finance:ListInvoices","description":"List invoices","resourcePattern":null,"sandboxRestriction":"none"}],"totalCount":1}"#,
            invalid_body: r#"{"actions":[{"id":"act_123456789012","platformId":null,"sharedWithDescendants":false,"ownerTenantId":null,"context":"finance","name":"ListInvoices","action":"finance:ListInvoices","description":"List invoices","resourcePattern":null,"sandboxRestriction":"none"}],"totalCount":1}"#,
            missing_field: "fullName",
            forbidden_output: None,
        },
    ]);
}
