mod response_contract_support;

use response_contract_support::{run_cases, run_value_case, Case};

const OPERATOR_VALID: &str = r#"[{"id":"tn_operator1234567890","name":"Operator","operatorName":"operator-one","platformId":"tn_platform123456789"}]"#;
const ACTION_VALID: &str = r#"{"actions":[{"id":"act_123456789012","platformId":null,"sharedWithDescendants":false,"ownerTenantId":null,"context":"finance","name":"ListInvoices","fullName":"finance:ListInvoices","description":"List invoices","resourcePattern":null,"sandboxRestriction":"none"}],"totalCount":1}"#;

#[test]
fn auth_responses_use_exact_api_fields() {
    run_cases(&[
        Case {
            label: "operator",
            args: &["org", "operators", "list", "--json"],
            path: "/v1/auth/operators/by-user",
            valid_body: OPERATOR_VALID,
            invalid_body: r#"[{"id":"tn_operator1234567890","name":"Operator","alias":"operator-one","platformId":"tn_platform123456789"}]"#,
            missing_field: "operatorName",
            forbidden_output: None,
        },
        Case {
            label: "policy action",
            args: &["org", "policies", "actions", "--json"],
            path: "/v1/auth/actions",
            valid_body: ACTION_VALID,
            invalid_body: r#"{"actions":[{"id":"act_123456789012","platformId":null,"sharedWithDescendants":false,"ownerTenantId":null,"context":"finance","name":"ListInvoices","action":"finance:ListInvoices","description":"List invoices","resourcePattern":null,"sandboxRestriction":"none"}],"totalCount":1}"#,
            missing_field: "fullName",
            forbidden_output: None,
        },
    ]);
}

#[test]
#[ignore = "run by the non-required Response contract workflow"]
fn auth_response_values_propagate_to_json_output() {
    run_value_case(
        "operator values",
        &["org", "operators", "list", "--json"],
        "/v1/auth/operators/by-user",
        OPERATOR_VALID,
        &[
            "tn_operator1234567890",
            "Operator",
            "operator-one",
            "tn_platform123456789",
        ],
    );
    run_value_case(
        "policy action values",
        &["org", "policies", "actions", "--json"],
        "/v1/auth/actions",
        ACTION_VALID,
        &[
            "act_123456789012",
            "finance",
            "ListInvoices",
            "finance:ListInvoices",
            "List invoices",
            "none",
        ],
    );
}
