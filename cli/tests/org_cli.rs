use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;

use tempfile::TempDir;

const TENANT_ID: &str = "tn_test1234567890";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut cmd = Command::new(bin());
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", TENANT_ID)
        .env_remove("TACHYON_API_KEY")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE");
    cmd
}

struct MockResponse {
    status: &'static str,
    body: &'static str,
}

fn start_server(
    responses: Vec<MockResponse>,
) -> (String, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut requests = Vec::with_capacity(responses.len());
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            requests.push(String::from_utf8_lossy(&buf[..n]).to_string());

            let raw_response = if response.body.is_empty() {
                format!(
                    "HTTP/1.1 {}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
                    response.status
                )
            } else {
                format!(
                    "HTTP/1.1 {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    response.status,
                    response.body.len(),
                    response.body
                )
            };
            stream.write_all(raw_response.as_bytes()).unwrap();
        }
        tx.send(requests).unwrap();
    });
    (url, rx, handle)
}

fn run_org(home: &Path, api_url: String, args: &[&str]) -> Output {
    isolated_command(home)
        .env("TACHYON_API_URL", api_url)
        .args(args)
        .output()
        .expect("run tachyon org command")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_decode_failure(output: &Output, missing_field: &str) {
    assert!(
        !output.status.success(),
        "response with a mismatched field unexpectedly decoded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(missing_field),
        "missing {missing_field:?} decode error; stderr was:\n{stderr}"
    );
}

fn finish_requests(rx: mpsc::Receiver<Vec<String>>, handle: thread::JoinHandle<()>) -> Vec<String> {
    let requests = rx.recv().unwrap();
    handle.join().unwrap();
    requests
}

fn assert_tenant_request(request: &str, request_line: &str) {
    assert!(request.starts_with(request_line), "request was:\n{request}");
    assert!(
        request.contains(&format!("x-operator-id: {TENANT_ID}")),
        "request was:\n{request}"
    );
}

#[test]
fn service_accounts_list_sends_tenant_query_and_decodes_openapi_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"serviceAccounts":[{"id":"sa_123456789012","tenantId":"tn_test1234567890","name":"inventory","createdAt":"2026-08-01T00:00:00Z"}]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "service-accounts", "list", "--json"],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts?operator_id=tn_test1234567890 ",
    );
    let accounts: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("service accounts json");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0]["id"], "sa_123456789012");
}

#[test]
fn service_accounts_list_rejects_wrong_item_fields() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"serviceAccounts":[{"id":"sa_123456789012","name":"legacy","description":"old field","createdAt":"2026-08-01T00:00:00Z"}]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "service-accounts", "list", "--json"],
    );
    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts?operator_id=tn_test1234567890 ",
    );
    assert_decode_failure(&output, "tenantId");
}

#[test]
fn service_accounts_get_sends_tenant_query_and_header() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"id":"sa_123456789012","tenantId":"tn_test1234567890","name":"inventory","createdAt":"2026-08-01T00:00:00Z"}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "service-accounts",
            "get",
            "sa_123456789012",
            "--json",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts/sa_123456789012?operator_id=tn_test1234567890 ",
    );
}

#[test]
fn service_account_api_keys_sends_tenant_query_and_decodes_openapi_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"apiKeys":[{"id":"key_123456789012","serviceAccountId":"sa_123456789012","name":"audit","value":"pk_****","createdAt":"2026-08-01T00:00:00Z","expiresAt":null}]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "service-accounts",
            "api-keys",
            "sa_123456789012",
            "--json",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts/sa_123456789012/api-keys?operator_id=tn_test1234567890 ",
    );
    let keys: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("api keys json");
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0]["id"], "key_123456789012");
}

#[test]
fn service_account_api_keys_reject_wrong_item_fields() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"apiKeys":[{"id":"key_123456789012","serviceAccountId":"sa_123456789012","name":"legacy","prefix":"pk_legacy","createdAt":"2026-08-01T00:00:00Z","expiresAt":null}]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "service-accounts",
            "api-keys",
            "sa_123456789012",
            "--json",
        ],
    );
    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts/sa_123456789012/api-keys?operator_id=tn_test1234567890 ",
    );
    assert_decode_failure(&output, "value");
}

#[test]
fn api_key_name_resolution_uses_tenant_query() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        MockResponse {
            status: "200 OK",
            body: r#"{"serviceAccounts":[{"id":"sa_123456789012","tenantId":"tn_test1234567890","name":"inventory","createdAt":"2026-08-01T00:00:00Z"}]}"#,
        },
        MockResponse {
            status: "200 OK",
            body: r#"{"apiKeys":[]}"#,
        },
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["api-key", "list", "inventory", "--json"],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 2);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts?operator_id=tn_test1234567890 ",
    );
    assert_tenant_request(
        &requests[1],
        "GET /v1/auth/service-accounts/sa_123456789012/api-keys?operator_id=tn_test1234567890 ",
    );
}

#[test]
fn users_list_sends_tenant_query_and_decodes_openapi_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"users":[{"id":"us_123456789012","name":"Member","email":"member@example.invalid","role":"member","tenants":["tn_test1234567890"],"status":"active","createdAt":"2026-08-01T00:00:00Z","expiresAt":null}]}"#,
    }]);

    let output = run_org(tmp.path(), api_url, &["org", "users", "list", "--json"]);
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/users?operator_id=tn_test1234567890 ",
    );
    let users: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).expect("users json");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["id"], "us_123456789012");
    assert_eq!(users[0]["name"], "Member");
}

#[test]
fn users_list_rejects_wrong_user_field_names() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"users":[{"id":"us_123456789012","username":"member","email":"member@example.invalid","role":"member","tenants":["tn_test1234567890"],"state":"active","createdAt":"2026-08-01T00:00:00Z","expiresAt":null}]}"#,
    }]);

    let output = run_org(tmp.path(), api_url, &["org", "users", "list", "--json"]);
    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/users?operator_id=tn_test1234567890 ",
    );
    assert_decode_failure(&output, "status");
}

#[test]
fn users_get_decodes_exact_contract_and_rejects_wrong_fields() {
    let valid = r#"{"id":"us_123456789012","email":"member@example.invalid","name":"Member","role":"member","tenants":["tn_test1234567890"]}"#;
    let invalid = r#"{"id":"us_123456789012","email":"member@example.invalid","username":"member","role_name":"member","tenants":["tn_test1234567890"]}"#;

    for (body, succeeds) in [(valid, true), (invalid, false)] {
        let tmp = TempDir::new().unwrap();
        let (api_url, rx, handle) = start_server(vec![MockResponse {
            status: "200 OK",
            body,
        }]);
        let output = run_org(
            tmp.path(),
            api_url,
            &["org", "users", "get", "us_123456789012", "--json"],
        );
        let requests = finish_requests(rx, handle);
        assert_tenant_request(&requests[0], "GET /v1/auth/users/us_123456789012 ");
        if succeeds {
            assert_success(&output);
        } else {
            assert_decode_failure(&output, "role");
        }
    }
}

#[test]
fn operators_decode_exact_contract_and_reject_alias_field() {
    let valid = r#"[{"id":"tn_operator1234567890","name":"Operator","operatorName":"operator-one","platformId":"tn_platform123456789"}]"#;
    let invalid = r#"[{"id":"tn_operator1234567890","name":"Operator","alias":"operator-one","platformId":"tn_platform123456789"}]"#;

    for (body, succeeds) in [(valid, true), (invalid, false)] {
        let tmp = TempDir::new().unwrap();
        let (api_url, rx, handle) = start_server(vec![MockResponse {
            status: "200 OK",
            body,
        }]);
        let output = run_org(tmp.path(), api_url, &["org", "operators", "list", "--json"]);
        let requests = finish_requests(rx, handle);
        assert_tenant_request(&requests[0], "GET /v1/auth/operators/by-user ");
        if succeeds {
            assert_success(&output);
        } else {
            assert_decode_failure(&output, "operatorName");
        }
    }
}

#[test]
fn policies_list_decodes_camel_case_api_contract() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"policies":[{"id":"pol_123456789012","name":"finance:Accountant","description":"Finance role","isSystem":false,"tenantId":"tn_test1234567890","sharedWithDescendants":true,"ownerTenantId":null,"createdAt":"2026-08-01T00:00:00Z","updatedAt":"2026-08-02T00:00:00Z"}],"totalCount":1}"#,
    }]);

    let output = run_org(tmp.path(), api_url, &["org", "policies", "list", "--json"]);
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "GET /v1/auth/policies ");
    let policies: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("policies json");
    assert_eq!(policies[0]["name"], "finance:Accountant");
    assert_eq!(policies[0]["sharedWithDescendants"], true);
}

#[test]
fn policies_list_rejects_wrong_policy_field_names() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"policies":[{"id":"pol_123456789012","name":"finance:Accountant","description":null,"is_system":false,"tenantId":null,"sharedWithDescendants":false,"ownerTenantId":null,"createdAt":"2026-08-01T00:00:00Z","updatedAt":"2026-08-02T00:00:00Z"}],"totalCount":1}"#,
    }]);

    let output = run_org(tmp.path(), api_url, &["org", "policies", "list", "--json"]);
    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "GET /v1/auth/policies ");
    assert_decode_failure(&output, "isSystem");
}

#[test]
fn policies_list_rejects_wrong_envelope_field_names() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"policies":[],"total_count":0}"#,
    }]);

    let output = run_org(tmp.path(), api_url, &["org", "policies", "list", "--json"]);
    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "GET /v1/auth/policies ");
    assert_decode_failure(&output, "totalCount");
}

#[test]
fn user_policies_decode_policy_ids_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"policyIds":["pol_123456789012"]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "users", "policies", "us_123456789012", "--json"],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "GET /v1/auth/users/us_123456789012/policies ");
    let policy_ids: Vec<String> = serde_json::from_slice(&output.stdout).expect("policy ids json");
    assert_eq!(policy_ids, ["pol_123456789012"]);
}

#[test]
fn user_policies_reject_wrong_policy_ids_field_name() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"policy_ids":["pol_123456789012"]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "users", "policies", "us_123456789012", "--json"],
    );
    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "GET /v1/auth/users/us_123456789012/policies ");
    assert_decode_failure(&output, "policyIds");
}

#[test]
fn policy_mappings_send_filters_and_decode_api_contract() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"mappings":[{"userId":"us_123456789012","tenantId":"tn_test1234567890","policyId":"pol_123456789012","resourceScope":"resource-scope","assignedAt":"2026-08-01T00:00:00Z"}]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "policies",
            "mappings",
            "--resource-scope",
            "resource-scope",
            "--json",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/user-policy-mappings?tenantId=tn_test1234567890&resourceScope=resource-scope ",
    );
    let mappings: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("policy mappings json");
    assert_eq!(mappings[0]["policyId"], "pol_123456789012");
    assert_eq!(mappings[0]["resourceScope"], "resource-scope");
}

#[test]
fn policy_mappings_reject_wrong_mapping_field_names() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"mappings":[{"user_id":"us_123456789012","tenantId":"tn_test1234567890","policyId":"pol_123456789012","resourceScope":null,"assignedAt":"2026-08-01T00:00:00Z"}]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "policies",
            "mappings",
            "--resource-scope",
            "resource-scope",
            "--json",
        ],
    );
    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/user-policy-mappings?tenantId=tn_test1234567890&resourceScope=resource-scope ",
    );
    assert_decode_failure(&output, "userId");
}

#[test]
fn policy_actions_decode_full_name_and_reject_legacy_action_field() {
    let valid_body = r#"{"actions":[{"id":"act_123456789012","platformId":null,"sharedWithDescendants":false,"ownerTenantId":null,"context":"finance","name":"ListInvoices","fullName":"finance:ListInvoices","description":"List invoices","resourcePattern":null,"sandboxRestriction":"none"}],"totalCount":1}"#;
    let invalid_body = r#"{"actions":[{"id":"act_123456789012","platformId":null,"sharedWithDescendants":false,"ownerTenantId":null,"context":"finance","name":"ListInvoices","action":"finance:ListInvoices","description":"List invoices","resourcePattern":null,"sandboxRestriction":"none"}],"totalCount":1}"#;

    for (body, succeeds) in [(valid_body, true), (invalid_body, false)] {
        let tmp = TempDir::new().unwrap();
        let (api_url, rx, handle) = start_server(vec![MockResponse {
            status: "200 OK",
            body,
        }]);
        let output = run_org(
            tmp.path(),
            api_url,
            &["org", "policies", "actions", "--json"],
        );
        let requests = finish_requests(rx, handle);
        assert_tenant_request(&requests[0], "GET /v1/auth/actions ");
        if succeeds {
            assert_success(&output);
            let actions: Vec<serde_json::Value> =
                serde_json::from_slice(&output.stdout).expect("actions json");
            assert_eq!(actions[0]["fullName"], "finance:ListInvoices");
        } else {
            assert_decode_failure(&output, "fullName");
        }
    }
}

#[test]
fn policies_delete_204_reports_success() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "204 No Content",
        body: "",
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "policies", "delete", "pol_123456789012"],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "DELETE /v1/auth/policies/pol_123456789012 ");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Policy pol_123456789012 deleted.\n"
    );
}

#[test]
fn policies_delete_legacy_global_403_is_not_reported_as_success() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "403 Forbidden",
        body: r#"{"code":"FORBIDDEN","message":"Forbidden: Global custom policies can only be deleted by a system executor"}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "policies", "delete", "pol_legacyglobal"],
    );

    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "DELETE /v1/auth/policies/pol_legacyglobal ");
    assert!(!output.status.success(), "403 must fail the command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("deleted"), "stdout was:\n{stdout}");
    assert!(stderr.contains("403 Forbidden"), "stderr was:\n{stderr}");
    assert!(
        stderr.contains("Global custom policies can only be deleted by a system executor"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn policies_delete_referenced_409_preserves_per_reference_counts() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "409 Conflict",
        body: r#"{"code":"CONFLICT","message":"Conflict: Policy is in use (userMappings=2, serviceAccountMappings=1, tenantOverrides=3); detach all references before deletion"}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "policies", "delete", "pol_referenced123"],
    );

    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "DELETE /v1/auth/policies/pol_referenced123 ");
    assert!(!output.status.success(), "409 must fail the command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("deleted"), "stdout was:\n{stdout}");
    assert!(stderr.contains("409 Conflict"), "stderr was:\n{stderr}");
    for count in [
        "userMappings=2",
        "serviceAccountMappings=1",
        "tenantOverrides=3",
    ] {
        assert!(
            stderr.contains(count),
            "missing {count}; stderr was:\n{stderr}"
        );
    }
}
