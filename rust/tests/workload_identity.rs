#![cfg(feature = "workload-identity")]

use tachyon_sdk::workload_identity::{
    exchange_token, github_actions_id_token_url, SubjectTokenSource, TokenExchangeRequest,
    WorkloadIdentityConfig, WorkloadIdentityCredentials, WorkloadIdentityError, DEFAULT_AUDIENCE,
    TOKEN_EXCHANGE_PATH,
};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config(api_url: &str, subject_token: SubjectTokenSource) -> WorkloadIdentityConfig {
    WorkloadIdentityConfig {
        api_url: api_url.to_string(),
        app_id: "app_01test".into(),
        operator_id: "tn_01test".into(),
        environment: "production".into(),
        action: Some("compute:CreateDeployment".into()),
        subject_token,
        ttl_seconds: Some(300),
    }
}

fn exchange_response(token: &str, expires_in: i64) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "tokenType": "Bearer",
        "accessToken": token,
        "expiresIn": expires_in,
    }))
}

#[tokio::test]
async fn exchange_posts_camel_case_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(TOKEN_EXCHANGE_PATH))
        .and(body_json(serde_json::json!({
            "appId": "app_01test",
            "operatorId": "tn_01test",
            "environment": "production",
            "action": "compute:CreateDeployment",
            "subjectToken": "oidc-jwt",
            "ttlSeconds": 300,
        })))
        .respond_with(exchange_response("tachyon-jwt", 300))
        .expect(1)
        .mount(&server)
        .await;

    let request = TokenExchangeRequest {
        app_id: "app_01test".into(),
        operator_id: "tn_01test".into(),
        environment: "production".into(),
        action: Some("compute:CreateDeployment".into()),
        subject_token: "oidc-jwt".into(),
        ttl_seconds: Some(300),
    };
    let response = exchange_token(&reqwest::Client::new(), &server.uri(), &request)
        .await
        .unwrap();
    assert_eq!(response.access_token, "tachyon-jwt");
    assert_eq!(response.expires_in, 300);
    assert!(!format!("{request:?}{response:?}").contains("jwt"));
}

#[tokio::test]
async fn exchange_omits_ttl_when_unset_and_reports_status() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(TOKEN_EXCHANGE_PATH))
        .respond_with(ResponseTemplate::new(403).set_body_string(r#"{"message":"denied"}"#))
        .mount(&server)
        .await;

    let mut cfg = config(&server.uri(), SubjectTokenSource::Literal("oidc".into()));
    cfg.ttl_seconds = None;
    let err = cfg.exchange(&reqwest::Client::new()).await.unwrap_err();
    match err {
        WorkloadIdentityError::Status { status, body, .. } => {
            assert_eq!(status, 403);
            assert!(body.contains("denied"));
        }
        other => panic!("unexpected error: {other}"),
    }
    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert!(body.get("ttlSeconds").is_none());
}

#[tokio::test]
async fn github_actions_source_requests_audience_with_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/token"))
        .and(query_param("api-version", "2.0"))
        .and(query_param("audience", DEFAULT_AUDIENCE))
        .and(header("authorization", "Bearer gh-request-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": "gh-oidc"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(TOKEN_EXCHANGE_PATH))
        .and(wiremock::matchers::body_partial_json(
            serde_json::json!({"subjectToken": "gh-oidc"}),
        ))
        .respond_with(exchange_response("tachyon-jwt", 900))
        .expect(1)
        .mount(&server)
        .await;

    let request_url = format!("{}/token?api-version=2.0", server.uri());
    let source = SubjectTokenSource::github_actions_from_lookup(None, |name| match name {
        "ACTIONS_ID_TOKEN_REQUEST_URL" => Some(request_url.clone()),
        "ACTIONS_ID_TOKEN_REQUEST_TOKEN" => Some("gh-request-token".into()),
        _ => None,
    })
    .unwrap();
    assert!(!format!("{source:?}").contains("gh-request-token"));

    let credentials =
        WorkloadIdentityCredentials::new(reqwest::Client::new(), config(&server.uri(), source));
    assert_eq!(credentials.access_token().await.unwrap(), "tachyon-jwt");
}

#[test]
fn github_actions_source_requires_both_variables() {
    assert!(
        SubjectTokenSource::github_actions_from_lookup(None, |name| {
            (name == "ACTIONS_ID_TOKEN_REQUEST_URL").then(|| "https://example.test/x?a=1".into())
        })
        .is_none()
    );
}

#[test]
fn github_actions_url_appends_audience() {
    let url =
        github_actions_id_token_url("https://example.test/x?api-version=2.0", "aud one").unwrap();
    assert_eq!(
        url.as_str(),
        "https://example.test/x?api-version=2.0&audience=aud+one"
    );
}

#[tokio::test]
async fn credentials_cache_until_near_expiry_and_refresh_on_demand() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(TOKEN_EXCHANGE_PATH))
        .respond_with(exchange_response("long-lived", 900))
        .mount(&server)
        .await;

    let dir = tempdir();
    let token_file = dir.join("token");
    std::fs::write(&token_file, "file-oidc\n").unwrap();
    let credentials = WorkloadIdentityCredentials::new(
        reqwest::Client::new(),
        config(&server.uri(), SubjectTokenSource::File(token_file)),
    );

    assert_eq!(credentials.access_token().await.unwrap(), "long-lived");
    assert_eq!(credentials.access_token().await.unwrap(), "long-lived");
    assert_eq!(server.received_requests().await.unwrap().len(), 1);

    credentials.refresh().await.unwrap();
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    let body: serde_json::Value = serde_json::from_slice(&requests[1].body).unwrap();
    assert_eq!(body["subjectToken"], "file-oidc");
}

#[tokio::test]
async fn credentials_re_exchange_when_token_is_already_expired() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(TOKEN_EXCHANGE_PATH))
        .respond_with(exchange_response("expired", 0))
        .mount(&server)
        .await;

    let credentials = WorkloadIdentityCredentials::new(
        reqwest::Client::new(),
        config(&server.uri(), SubjectTokenSource::Literal("oidc".into())),
    );
    credentials.access_token().await.unwrap();
    credentials.access_token().await.unwrap();
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tachyon-sdk-wif-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn exchange_omits_action_when_none() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(TOKEN_EXCHANGE_PATH))
        .respond_with(exchange_response("tachyon-jwt", 300))
        .mount(&server)
        .await;

    let mut cfg = config(&server.uri(), SubjectTokenSource::Literal("oidc".into()));
    cfg.action = None;
    cfg.exchange(&reqwest::Client::new()).await.unwrap();

    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert!(body.get("action").is_none(), "{body}");
    assert_eq!(body["appId"], "app_01test");
}
