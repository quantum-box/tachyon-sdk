//! End-to-end checks for `tachyon data` against a stub HTTP server.
//!
//! The Data API has two shapes that a unit test cannot cover: a query binds
//! an alias to a dataset *version*, so the CLI has to describe the dataset
//! first; and a refused query still answers `202`, carrying the refusal in
//! the body. Both are asserted here against the real request the CLI sends.

use std::io::{BufRead, BufReader, Read, Write};
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

/// One request the stub server saw.
struct Request {
    start_line: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    fn assert_start_line(&self, expected: &str) {
        assert!(
            self.start_line.starts_with(&format!("{expected} ")),
            "expected `{expected}`, request was `{}`",
            self.start_line
        );
    }
}

/// A canned response: status code and JSON body.
struct Reply(u16, &'static str);

/// Serve `replies` in order, one per connection, and hand back what was sent.
fn start_server(replies: Vec<Reply>) -> (String, mpsc::Receiver<Vec<Request>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut seen = Vec::with_capacity(replies.len());
        for Reply(status, body) in replies {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream);

            let mut start_line = String::new();
            reader.read_line(&mut start_line).unwrap();
            let mut headers = Vec::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if let Some((key, value)) = line.split_once(':') {
                    headers.push((key.trim().to_string(), value.trim().to_string()));
                }
            }

            // Read exactly the advertised body, so a multipart upload is
            // captured whole rather than truncated at a buffer boundary.
            let length: usize = headers
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                .and_then(|(_, value)| value.parse().ok())
                .unwrap_or(0);
            let mut body_bytes = vec![0_u8; length];
            if length > 0 {
                reader.read_exact(&mut body_bytes).unwrap();
            }

            seen.push(Request {
                start_line: start_line.trim_end().to_string(),
                headers,
                body: String::from_utf8_lossy(&body_bytes).to_string(),
            });

            let reason = if status == 202 { "Accepted" } else { "OK" };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\ncontent-type: application/json\r\n\
                 content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let mut stream = reader.into_inner();
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        tx.send(seen).unwrap();
    });
    (url, rx)
}

fn run(home: &Path, api_url: &str, args: &[&str]) -> Output {
    let mut cmd = Command::new(bin());
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_API_URL", api_url)
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_TENANT_ID");
    cmd.args(["--tenant-id", TENANT_ID])
        .args(args)
        .output()
        .expect("run tachyon data command")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const DATASET_PAGE: &str = r#"{"items":[{"id":"ds_1","name":"sales","description":"synthetic","latest_version":"dsv_1","updated_at":"2026-09-19T02:00:00Z"}],"total_count":1}"#;

const DATASET_DESCRIPTION: &str = r#"{"dataset":{"id":"ds_1","name":"sales","latest_version":"dsv_1","updated_at":"2026-09-19T02:00:00Z"},"version":"dsv_1","schema":{"columns":[{"name":"region","type":"string","nullable":false},{"name":"amount","type":"int64","nullable":false}]},"sql_dialect":{"name":"mysql","version":"1"},"access":{"purpose":"unspecified","row_filtered":false,"access_revision":"rev_1"}}"#;

const JOB_QUEUED: &str = r#"{"job_id":"qj_1","state":"queued","bindings":{"sales":"dsv_1"},"read_versions":{},"truncated":false,"submitted_at":"2026-09-19T02:00:00Z","purpose":"tenant_analytics","access_revision":"rev_1"}"#;

const JOB_SUCCEEDED: &str = r#"{"job_id":"qj_1","state":"succeeded","bindings":{"sales":"dsv_1"},"read_versions":{"sales":"dsv_1"},"sql_dialect":{"name":"mysql","version":"1"},"row_count":2,"truncated":false,"submitted_at":"2026-09-19T02:00:00Z","finished_at":"2026-09-19T02:00:02Z","purpose":"tenant_analytics","access_revision":"rev_1"}"#;

const JOB_RESULT: &str = r#"{"job_id":"qj_1","schema":{"columns":[{"name":"region","type":"string","nullable":true},{"name":"total","type":"decimal","nullable":true}]},"rows":[["east","150"],["west","250"]],"total_rows":2,"truncated":false,"read_versions":{"sales":"dsv_1"}}"#;

#[test]
fn dataset_list_renders_a_table_and_sends_tenant_headers() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![Reply(200, DATASET_PAGE)]);

    let output = run(home.path(), &url, &["data", "dataset", "list"]);
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ds_1"), "{stdout}");
    assert!(stdout.contains("sales"), "{stdout}");
    assert!(stdout.contains("1 shown, 1 total"), "{stdout}");

    let requests = rx.recv().unwrap();
    requests[0].assert_start_line("GET /data/v1/datasets");
    assert_eq!(requests[0].header("x-operator-id"), Some(TENANT_ID));
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer test-token")
    );
    // No purpose is declared for a listing; the endpoint does not read it.
    assert_eq!(requests[0].header("x-data-purpose"), None);
}

#[test]
fn dataset_list_passes_limit_and_offset_through() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![Reply(200, DATASET_PAGE)]);

    let output = run(
        home.path(),
        &url,
        &["data", "dataset", "list", "--limit", "5", "--offset", "10"],
    );
    assert_success(&output);

    let requests = rx.recv().unwrap();
    requests[0].assert_start_line("GET /data/v1/datasets?limit=5&offset=10");
}

#[test]
fn dataset_create_uploads_multipart_with_the_purpose_header() {
    let home = TempDir::new().unwrap();
    let csv = home.path().join("sales.csv");
    std::fs::write(&csv, "region,amount\neast,100\n").unwrap();
    let (url, rx) = start_server(vec![Reply(
        200,
        r#"{"dataset_id":"ds_1","version_id":"dsv_1","row_count":1}"#,
    )]);

    let output = run(
        home.path(),
        &url,
        &[
            "data",
            "dataset",
            "create",
            "--name",
            "synthetic_sales",
            "--file",
            csv.to_str().unwrap(),
            "--description",
            "a description",
            "--purpose",
            "tenant_analytics",
        ],
    );
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ds_1"), "{stdout}");
    assert!(stdout.contains("dsv_1"), "{stdout}");

    let requests = rx.recv().unwrap();
    let request = &requests[0];
    request.assert_start_line("POST /data/v1/datasets");
    assert_eq!(request.header("x-data-purpose"), Some("tenant_analytics"));
    assert!(
        request
            .header("content-type")
            .unwrap_or_default()
            .starts_with("multipart/form-data"),
        "content-type was {:?}",
        request.header("content-type")
    );
    assert!(request.body.contains("name=\"name\""), "{}", request.body);
    assert!(request.body.contains("synthetic_sales"), "{}", request.body);
    assert!(
        request.body.contains("name=\"description\""),
        "{}",
        request.body
    );
    assert!(request.body.contains("a description"), "{}", request.body);
    assert!(
        request.body.contains("filename=\"sales.csv\""),
        "{}",
        request.body
    );
    assert!(request.body.contains("east,100"), "{}", request.body);
}

#[test]
fn dataset_create_omits_the_purpose_header_when_not_asked() {
    let home = TempDir::new().unwrap();
    let csv = home.path().join("sales.csv");
    std::fs::write(&csv, "region,amount\neast,100\n").unwrap();
    let (url, rx) = start_server(vec![Reply(
        200,
        r#"{"dataset_id":"ds_1","version_id":"dsv_1","row_count":1}"#,
    )]);

    let output = run(
        home.path(),
        &url,
        &[
            "data",
            "dataset",
            "create",
            "--name",
            "sales",
            "--file",
            csv.to_str().unwrap(),
        ],
    );
    assert_success(&output);

    let requests = rx.recv().unwrap();
    assert_eq!(requests[0].header("x-data-purpose"), None);
    assert!(
        !requests[0].body.contains("name=\"description\""),
        "{}",
        requests[0].body
    );
}

#[test]
fn dataset_get_prints_the_schema_and_dialect() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![Reply(200, DATASET_DESCRIPTION)]);

    let output = run(home.path(), &url, &["data", "dataset", "get", "ds_1"]);
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Version:     dsv_1"), "{stdout}");
    assert!(stdout.contains("mysql 1"), "{stdout}");
    assert!(stdout.contains("region"), "{stdout}");
    assert!(stdout.contains("int64"), "{stdout}");

    let requests = rx.recv().unwrap();
    requests[0].assert_start_line("GET /data/v1/datasets/ds_1");
}

#[test]
fn dataset_get_pins_a_version() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![Reply(200, DATASET_DESCRIPTION)]);

    let output = run(
        home.path(),
        &url,
        &["data", "dataset", "get", "ds_1", "--version", "dsv_1"],
    );
    assert_success(&output);

    let requests = rx.recv().unwrap();
    requests[0].assert_start_line("GET /data/v1/datasets/ds_1?version=dsv_1");
}

#[test]
fn query_run_resolves_a_dataset_bind_to_a_version_then_waits() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![
        Reply(200, DATASET_DESCRIPTION),
        Reply(202, JOB_QUEUED),
        Reply(200, JOB_SUCCEEDED),
        Reply(200, JOB_RESULT),
    ]);

    let output = run(
        home.path(),
        &url,
        &[
            "data",
            "query",
            "run",
            "--sql",
            "SELECT region, SUM(amount) AS total FROM sales GROUP BY region",
            "--bind",
            "sales=ds_1",
            "--purpose",
            "tenant_analytics",
            "--wait",
            "--interval-secs",
            "1",
        ],
    );
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("east"), "{stdout}");
    assert!(stdout.contains("150"), "{stdout}");
    assert!(stdout.contains("west"), "{stdout}");
    assert!(stdout.contains("2 of 2 rows"), "{stdout}");
    assert!(stdout.contains("Provenance:"), "{stdout}");
    assert!(stdout.contains("query job:       qj_1"), "{stdout}");
    assert!(stdout.contains("sales = dsv_1"), "{stdout}");
    assert!(stdout.contains("sql dialect:     mysql 1"), "{stdout}");

    let requests = rx.recv().unwrap();
    assert_eq!(requests.len(), 4);

    // 1. The dataset is described so the bind resolves to a concrete version.
    requests[0].assert_start_line("GET /data/v1/datasets/ds_1");
    assert_eq!(
        requests[0].header("x-data-purpose"),
        Some("tenant_analytics")
    );

    // 2. The submitted body is flat and binds the version, not the dataset.
    requests[1].assert_start_line("POST /data/v1/query-jobs");
    let submitted: serde_json::Value = serde_json::from_str(&requests[1].body).unwrap();
    assert_eq!(
        submitted,
        serde_json::json!({
            "sql": "SELECT region, SUM(amount) AS total FROM sales GROUP BY region",
            "bindings": {"sales": "dsv_1"},
        })
    );
    assert_eq!(
        requests[1].header("x-data-purpose"),
        Some("tenant_analytics")
    );

    // 3/4. The job is polled, then its result is read.
    requests[2].assert_start_line("GET /data/v1/query-jobs/qj_1");
    requests[3].assert_start_line("GET /data/v1/query-jobs/qj_1/result");
}

#[test]
fn query_run_binds_a_version_id_without_describing_the_dataset() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![Reply(202, JOB_SUCCEEDED), Reply(200, JOB_RESULT)]);

    let output = run(
        home.path(),
        &url,
        &[
            "data",
            "query",
            "run",
            "--sql",
            "SELECT * FROM sales",
            "--bind",
            "sales=dsv_1",
            "--wait",
        ],
    );
    assert_success(&output);

    let requests = rx.recv().unwrap();
    assert_eq!(requests.len(), 2, "no describe call should be made");
    requests[0].assert_start_line("POST /data/v1/query-jobs");
}

#[test]
fn query_run_pins_the_version_named_after_the_at_sign() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![
        Reply(200, DATASET_DESCRIPTION),
        Reply(202, JOB_SUCCEEDED),
        Reply(200, JOB_RESULT),
    ]);

    let output = run(
        home.path(),
        &url,
        &[
            "data",
            "query",
            "run",
            "--sql",
            "SELECT * FROM sales",
            "--bind",
            "sales=ds_1@dsv_1",
            "--wait",
        ],
    );
    assert_success(&output);

    let requests = rx.recv().unwrap();
    requests[0].assert_start_line("GET /data/v1/datasets/ds_1?version=dsv_1");
}

#[test]
fn a_refused_query_is_an_error_even_though_the_submit_returned_202() {
    let home = TempDir::new().unwrap();
    let refused = r#"{"job_id":"qj_2","state":"failed","bindings":{"sales":"dsv_1"},"read_versions":{},"truncated":false,"failure":{"code":"TENANT_QUERY_CONCURRENCY","message":"too many open query jobs"},"submitted_at":"2026-09-19T02:00:00Z","finished_at":"2026-09-19T02:00:00Z","purpose":"unspecified"}"#;
    let (url, _rx) = start_server(vec![Reply(202, refused)]);

    let output = run(
        home.path(),
        &url,
        &[
            "data",
            "query",
            "run",
            "--sql",
            "SELECT * FROM sales",
            "--bind",
            "sales=dsv_1",
            "--wait",
        ],
    );
    assert!(
        !output.status.success(),
        "a refused query must exit non-zero\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("TENANT_QUERY_CONCURRENCY"), "{stderr}");
    assert!(stderr.contains("too many open query jobs"), "{stderr}");
    assert!(stderr.contains("about 30s"), "{stderr}");
}

#[test]
fn query_run_without_wait_reports_the_job_and_how_to_follow_it() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![Reply(202, JOB_QUEUED)]);

    let output = run(
        home.path(),
        &url,
        &[
            "data",
            "query",
            "run",
            "--sql",
            "SELECT * FROM sales",
            "--bind",
            "sales=dsv_1",
        ],
    );
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Query job: qj_1"), "{stdout}");
    assert!(stdout.contains("State:     queued"), "{stdout}");
    assert!(
        stdout.contains("tachyon data query get qj_1 --wait"),
        "{stdout}"
    );

    let requests = rx.recv().unwrap();
    assert_eq!(requests.len(), 1, "it must not poll without --wait");
}

#[test]
fn query_run_json_emits_job_provenance_and_result() {
    let home = TempDir::new().unwrap();
    let (url, _rx) = start_server(vec![Reply(202, JOB_SUCCEEDED), Reply(200, JOB_RESULT)]);

    let output = run(
        home.path(),
        &url,
        &[
            "data",
            "query",
            "run",
            "--sql",
            "SELECT * FROM sales",
            "--bind",
            "sales=dsv_1",
            "--wait",
            "--json",
        ],
    );
    assert_success(&output);

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("--json must emit one JSON document");
    assert_eq!(parsed["job"]["job_id"], "qj_1");
    assert_eq!(parsed["result"]["rows"][0][0], "east");
    assert_eq!(parsed["provenance"]["query_job_id"], "qj_1");
    assert_eq!(parsed["provenance"]["tenant_id"], TENANT_ID);
    assert_eq!(parsed["provenance"]["read_versions"]["sales"], "dsv_1");
    // sha256 of the SQL that ran, so the result can be cited against it.
    assert_eq!(
        parsed["provenance"]["sql_sha256"],
        "4c75abdcf100788aa39a2bee2ac341a218418dd23b8789c3f814c1ebb7de24c0"
    );
}

#[test]
fn query_get_follows_an_existing_job() {
    let home = TempDir::new().unwrap();
    let (url, rx) = start_server(vec![Reply(200, JOB_SUCCEEDED), Reply(200, JOB_RESULT)]);

    let output = run(
        home.path(),
        &url,
        &["data", "query", "get", "qj_1", "--wait"],
    );
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 of 2 rows"), "{stdout}");

    let requests = rx.recv().unwrap();
    requests[0].assert_start_line("GET /data/v1/query-jobs/qj_1");
    requests[1].assert_start_line("GET /data/v1/query-jobs/qj_1/result");
}

#[test]
fn query_get_omits_the_sql_digest_it_cannot_know() {
    let home = TempDir::new().unwrap();
    let (url, _rx) = start_server(vec![Reply(200, JOB_SUCCEEDED), Reply(200, JOB_RESULT)]);

    let output = run(
        home.path(),
        &url,
        &["data", "query", "get", "qj_1", "--wait", "--json"],
    );
    assert_success(&output);

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // `query get` did not submit the SQL, so it must not claim a digest for
    // it rather than reporting an empty one.
    assert!(
        parsed["provenance"].get("sql_sha256").is_none(),
        "provenance was {}",
        parsed["provenance"]
    );
    assert_eq!(parsed["provenance"]["query_job_id"], "qj_1");
}

#[test]
fn a_gate_404_with_no_body_says_the_flag_is_off() {
    let home = TempDir::new().unwrap();
    let (url, _rx) = start_server(vec![Reply(404, "")]);

    let output = run(home.path(), &url, &["data", "dataset", "list"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not enabled for tenant"), "{stderr}");
    assert!(stderr.contains("data_platform"), "{stderr}");
}

#[test]
fn a_bad_bind_fails_before_any_request_is_made() {
    let home = TempDir::new().unwrap();
    let output = run(
        home.path(),
        "http://127.0.0.1:1",
        &[
            "data",
            "query",
            "run",
            "--sql",
            "SELECT 1",
            "--bind",
            "no-equals-sign",
        ],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("NAME=DATASET_ID"), "{stderr}");
}

#[test]
fn a_query_with_no_bind_is_refused_locally() {
    let home = TempDir::new().unwrap();
    let output = run(
        home.path(),
        "http://127.0.0.1:1",
        &["data", "query", "run", "--sql", "SELECT 1"],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("must bind at least one dataset"),
        "{stderr}"
    );
}

#[test]
fn an_unknown_purpose_is_refused_by_the_parser() {
    let home = TempDir::new().unwrap();
    let csv = home.path().join("sales.csv");
    std::fs::write(&csv, "a\n1\n").unwrap();
    let output = run(
        home.path(),
        "http://127.0.0.1:1",
        &[
            "data",
            "dataset",
            "create",
            "--name",
            "sales",
            "--file",
            csv.to_str().unwrap(),
            "--purpose",
            "not_a_purpose",
        ],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tenant_analytics"), "{stderr}");
}
