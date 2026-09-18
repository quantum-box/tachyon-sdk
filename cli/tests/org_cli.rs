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

/// Read one HTTP request, including a `content-length` body that may arrive
/// in a later segment than the headers.
fn read_request(stream: &mut std::net::TcpStream) -> String {
    let mut raw = Vec::new();
    let mut buf = [0_u8; 8192];
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        raw.extend_from_slice(&buf[..n]);
        let text = String::from_utf8_lossy(&raw).to_string();
        let Some(header_end) = text.find("\r\n\r\n") else {
            continue;
        };
        let content_length = text[..header_end]
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.trim()
                    .eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())?
            })
            .unwrap_or(0);
        if raw.len() >= header_end + 4 + content_length {
            break;
        }
    }
    String::from_utf8_lossy(&raw).to_string()
}

/// The JSON body of a raw request captured by [`start_server`].
fn request_body(request: &str) -> serde_json::Value {
    let (_, body) = request
        .split_once("\r\n\r\n")
        .unwrap_or_else(|| panic!("request has no body:\n{request}"));
    serde_json::from_str(body).unwrap_or_else(|e| panic!("body {body:?} is not JSON: {e}"))
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
            requests.push(read_request(&mut stream));

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
fn operators_delete_200_reports_success() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"success":true}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "operators", "delete", "tn_target1234567890"],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "DELETE /v1/auth/operators/tn_target1234567890 ",
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Operator tn_target1234567890 deleted.\n"
    );
}

#[test]
fn operators_delete_403_is_not_reported_as_success() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "403 Forbidden",
        body: r#"{"code":"FORBIDDEN","message":"PermissionDenied: You do not have permission for this tenant"}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "operators", "delete", "tn_unrelated1234567"],
    );

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "DELETE /v1/auth/operators/tn_unrelated1234567 ",
    );
    assert!(!output.status.success(), "403 must fail the command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("deleted"), "stdout was:\n{stdout}");
    assert!(stderr.contains("403 Forbidden"), "stderr was:\n{stderr}");
    assert!(
        stderr.contains("You do not have permission for this tenant"),
        "stderr was:\n{stderr}"
    );
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

// --- org users attach-policy / detach-policy ---

const USER_ID: &str = "us_123456789012";
const POLICY_ID: &str = "pol_field1234567890";

const POLICIES_WITH_FIELD_STAFF: &str = r#"{"policies":[{"id":"pol_field1234567890","name":"field:staff","description":"Field staff","isSystem":false,"tenantId":"tn_test1234567890","sharedWithDescendants":false,"ownerTenantId":"tn_test1234567890","createdAt":"2026-09-01T00:00:00Z","updatedAt":"2026-09-01T00:00:00Z"}],"totalCount":1}"#;

const POLICIES_WITH_DUPLICATE_FIELD_ADMIN: &str = r#"{"policies":[{"id":"pol_owned1234567890","name":"field:admin","description":null,"isSystem":false,"tenantId":"tn_test1234567890","sharedWithDescendants":false,"ownerTenantId":"tn_test1234567890","createdAt":"2026-09-01T00:00:00Z","updatedAt":"2026-09-01T00:00:00Z"},{"id":"pol_parent123456789","name":"field:admin","description":null,"isSystem":false,"tenantId":"tn_parent123456789","sharedWithDescendants":true,"ownerTenantId":"tn_parent123456789","createdAt":"2026-09-01T00:00:00Z","updatedAt":"2026-09-01T00:00:00Z"}],"totalCount":2}"#;

const USER_BODY: &str = r#"{"id":"us_123456789012","email":"member@example.invalid","name":"Member","role":"member","tenants":["tn_test1234567890"]}"#;

fn ok(body: &'static str) -> MockResponse {
    MockResponse {
        status: "200 OK",
        body,
    }
}

#[test]
fn attach_policy_resolves_name_and_posts_the_grant() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        ok(POLICIES_WITH_FIELD_STAFF),
        ok(USER_BODY),
        ok(r#"{"policyIds":[]}"#),
        ok("{}"),
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            "field:staff",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], "GET /v1/auth/policies ");
    assert_tenant_request(&requests[1], &format!("GET /v1/auth/users/{USER_ID} "));
    assert_tenant_request(
        &requests[2],
        &format!("GET /v1/auth/users/{USER_ID}/policies "),
    );
    assert_tenant_request(&requests[3], "POST /v1/auth/user-policies/attach ");
    assert_eq!(
        request_body(&requests[3]),
        serde_json::json!({
            "userId": USER_ID,
            "policyId": POLICY_ID,
            "tenantId": TENANT_ID,
        }),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "field:staff",
        POLICY_ID,
        USER_ID,
        "member@example.invalid",
        TENANT_ID,
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected:?} in stdout:\n{stdout}"
        );
    }
}

#[test]
fn attach_policy_uses_the_scoped_endpoint_when_a_resource_scope_is_given() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        ok(POLICIES_WITH_FIELD_STAFF),
        ok(USER_BODY),
        ok(r#"{"policyIds":[]}"#),
        ok("{}"),
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            "field:staff",
            "--resource-scope",
            "trn:library:repo:rp_123456789012",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[3],
        "POST /v1/auth/user-policies/attach-with-scope ",
    );
    assert_eq!(
        request_body(&requests[3]),
        serde_json::json!({
            "userId": USER_ID,
            "policyId": POLICY_ID,
            "tenantId": TENANT_ID,
            "resourceScope": "trn:library:repo:rp_123456789012",
        }),
    );
}

#[test]
fn attach_policy_accepts_a_policy_id_without_listing_policies() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        ok(
            r#"{"id":"pol_field1234567890","name":"field:staff","description":null,"isSystem":false,"tenantId":"tn_test1234567890","sharedWithDescendants":false,"ownerTenantId":"tn_test1234567890","createdAt":"2026-09-01T00:00:00Z","updatedAt":"2026-09-01T00:00:00Z"}"#,
        ),
        ok(USER_BODY),
        ok(r#"{"policyIds":[]}"#),
        ok("{}"),
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            POLICY_ID,
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[0], &format!("GET /v1/auth/policies/{POLICY_ID} "));
    assert_tenant_request(&requests[3], "POST /v1/auth/user-policies/attach ");
}

#[test]
fn attach_policy_reports_a_policy_that_is_already_listed() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        ok(POLICIES_WITH_FIELD_STAFF),
        ok(USER_BODY),
        ok(r#"{"policyIds":["pol_field1234567890"]}"#),
        ok("{}"),
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            "field:staff",
        ],
    );
    assert_success(&output);
    finish_requests(rx, handle);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("already listed for this user"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn attach_policy_refuses_an_ambiguous_policy_name() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![ok(POLICIES_WITH_DUPLICATE_FIELD_ADMIN)]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            "field:admin",
        ],
    );

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1, "the grant must not be written");
    assert!(!output.status.success(), "an ambiguous name must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "ambiguous",
        "pol_owned1234567890",
        "pol_parent123456789",
        "tn_parent123456789",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in stderr:\n{stderr}"
        );
    }
}

#[test]
fn attach_policy_explains_an_unknown_policy_name() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![ok(r#"{"policies":[],"totalCount":0}"#)]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            "field:staff",
        ],
    );

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1, "the grant must not be written");
    assert!(!output.status.success(), "an unknown policy must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tachyon org policies list"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn attach_policy_explains_an_unknown_user_before_writing() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        ok(POLICIES_WITH_FIELD_STAFF),
        MockResponse {
            status: "404 Not Found",
            body: r#"{"code":"NOT_FOUND","message":"NotFound: User"}"#,
        },
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            "field:staff",
        ],
    );

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 2, "the grant must not be written");
    assert!(!output.status.success(), "an unknown user must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tachyon org users invite"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn attach_policy_403_names_the_delegation_rule() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        ok(POLICIES_WITH_FIELD_STAFF),
        ok(USER_BODY),
        ok(r#"{"policyIds":[]}"#),
        MockResponse {
            status: "403 Forbidden",
            body: r#"{"code":"FORBIDDEN","message":"Forbidden: policy does not belong to the authorized tenant scope"}"#,
        },
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            "field:staff",
        ],
    );

    finish_requests(rx, handle);
    assert!(!output.status.success(), "403 must fail the command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Attached policy"), "stdout was:\n{stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "403 Forbidden",
        "policy does not belong to the authorized tenant scope",
        "AdministratorAccess",
        "--profile",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in stderr:\n{stderr}"
        );
    }
}

#[test]
fn attach_policy_rejects_a_resource_scope_that_is_not_a_trn() {
    let tmp = TempDir::new().unwrap();
    let output = run_org(
        tmp.path(),
        "http://127.0.0.1:1".to_string(),
        &[
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            "field:staff",
            "--resource-scope",
            "library:repo:rp_123456789012",
        ],
    );

    assert!(!output.status.success(), "a non-TRN scope must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("trn:<service>:<resource-type>:<resource-id>"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn attach_policy_requires_a_tenant() {
    let tmp = TempDir::new().unwrap();
    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", "http://127.0.0.1:1")
        .env("TACHYON_TENANT_ID", "")
        .args([
            "org",
            "users",
            "attach-policy",
            USER_ID,
            "--policy",
            POLICY_ID,
        ])
        .output()
        .expect("run tachyon org users attach-policy");

    assert!(!output.status.success(), "a missing tenant must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--tenant-id"), "stderr was:\n{stderr}");
}

#[test]
fn detach_policy_posts_the_detach_and_reports_a_no_op() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        ok(POLICIES_WITH_FIELD_STAFF),
        ok(USER_BODY),
        ok(r#"{"policyIds":[]}"#),
        ok("{}"),
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "detach-policy",
            USER_ID,
            "--policy",
            "field:staff",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(&requests[3], "POST /v1/auth/user-policies/detach ");
    assert_eq!(
        request_body(&requests[3]),
        serde_json::json!({
            "userId": USER_ID,
            "policyId": POLICY_ID,
            "tenantId": TENANT_ID,
        }),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("was not listed for this user"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("Every scope of this policy"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn detach_policy_with_a_resource_scope_uses_the_scoped_endpoint() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        ok(POLICIES_WITH_FIELD_STAFF),
        ok(USER_BODY),
        ok(r#"{"policyIds":["pol_field1234567890"]}"#),
        ok("{}"),
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "users",
            "detach-policy",
            USER_ID,
            "--policy",
            "field:staff",
            "--resource-scope",
            "trn:library:repo:rp_123456789012",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[3],
        "POST /v1/auth/user-policies/detach-with-scope ",
    );
    assert_eq!(
        request_body(&requests[3])["resourceScope"],
        "trn:library:repo:rp_123456789012",
    );
}
