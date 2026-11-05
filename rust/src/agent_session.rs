//! Tachyon agent sessions (`/v1/llms/sessions`).
//!
//! Tachyon runs the agent loop and keeps the history; the app streams the
//! turn's events and answers the client-side tools the model calls.
//!
//! Hand-written alongside the generated `apis::agent_api`: that reads
//! `execute` as one JSON body instead of a stream, and its request model
//! drops fields such as `attachments`. Here request bodies pass through as
//! given and `execute` returns the open event stream.

use std::fmt;
use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use serde_json::{json, Value};

/// Why a session call failed.
#[derive(Debug)]
pub enum AgentSessionError {
    /// The session id is not an `as_...` id this client will put in a path.
    InvalidSession,
    /// The request did not reach Tachyon or its response could not be read.
    Unavailable(reqwest::Error),
    /// Tachyon refused the caller (401/403), e.g. a tenant whose plan has no
    /// AI. Retrying does not help.
    Unauthorized(StatusCode),
    /// Tachyon answered with another non-success status.
    Status(StatusCode),
    /// Tachyon answered successfully but not with what the call expects.
    InvalidResponse(&'static str),
}

impl fmt::Display for AgentSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSession => write!(f, "invalid agent session id"),
            Self::Unavailable(error) => write!(f, "agent session request failed: {error}"),
            Self::Unauthorized(status) => write!(f, "agent session call not authorized ({status})"),
            Self::Status(status) => write!(f, "agent session request failed ({status})"),
            Self::InvalidResponse(reason) => write!(f, "invalid agent session response: {reason}"),
        }
    }
}

impl std::error::Error for AgentSessionError {}

/// The API path of one session, after checking the id is an `as_...` id.
pub fn session_path(session_id: &str) -> Result<String, AgentSessionError> {
    if !session_id.starts_with("as_")
        || session_id.len() > 128
        || !session_id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(AgentSessionError::InvalidSession);
    }
    Ok(format!("/v1/llms/sessions/{session_id}"))
}

/// Calls the session API as one caller (bearer token and tenant).
#[derive(Clone)]
pub struct AgentSessionClient {
    base_path: String,
    client: Client,
    token: String,
    operator_id: String,
    platform_id: Option<String>,
    tool_result_retry_base: Duration,
}

/// Times a tool result is resent when Tachyon briefly reports the pending
/// call as missing.
const TOOL_RESULT_RETRIES: u32 = 5;

impl AgentSessionClient {
    pub fn new(
        base_path: impl Into<String>,
        token: impl Into<String>,
        operator_id: impl Into<String>,
    ) -> Self {
        Self {
            base_path: base_path.into().trim_end_matches('/').to_owned(),
            client: Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            token: token.into(),
            operator_id: operator_id.into(),
            platform_id: None,
            tool_result_retry_base: Duration::from_millis(250),
        }
    }

    /// Send `x-platform-id` with every call.
    pub fn with_platform_id(mut self, platform_id: Option<String>) -> Self {
        self.platform_id = platform_id;
        self
    }

    /// Use this HTTP client instead of the default one.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Wait this long before the first resend of a tool result; each later
    /// resend waits twice as long.
    pub fn with_tool_result_retry_base(mut self, delay: Duration) -> Self {
        self.tool_result_retry_base = delay;
        self
    }

    async fn post(
        &self,
        path: &str,
        body: &Value,
        accept: &str,
    ) -> Result<Response, reqwest::Error> {
        let mut request = self
            .client
            .post(format!("{}{path}", self.base_path))
            .header("accept", accept)
            .bearer_auth(&self.token)
            .header("x-operator-id", &self.operator_id)
            .json(body);
        if let Some(platform_id) = &self.platform_id {
            request = request.header("x-platform-id", platform_id);
        }
        request.send().await
    }

    fn check(response: Response) -> Result<Response, AgentSessionError> {
        let status = response.status();
        if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            return Err(AgentSessionError::Unauthorized(status));
        }
        if !status.is_success() {
            return Err(AgentSessionError::Status(status));
        }
        Ok(response)
    }

    /// Start a session. Returns Tachyon's response, which carries a valid
    /// `session_id`.
    pub async fn create_session(&self, body: &Value) -> Result<Value, AgentSessionError> {
        let response = self
            .post("/v1/llms/sessions", body, "application/json")
            .await
            .map_err(AgentSessionError::Unavailable)?;
        let value: Value = Self::check(response)?
            .json()
            .await
            .map_err(AgentSessionError::Unavailable)?;
        match value["session_id"].as_str() {
            Some(id) if session_path(id).is_ok() => Ok(value),
            _ => Err(AgentSessionError::InvalidResponse("no session id")),
        }
    }

    /// Run one turn. Returns the open server-sent event stream; read it with
    /// `bytes_stream()` and `parse_sse_record`.
    pub async fn execute(
        &self,
        session_id: &str,
        body: &Value,
    ) -> Result<Response, AgentSessionError> {
        let path = format!("{}/agent/execute", session_path(session_id)?);
        let response = self
            .post(&path, body, "text/event-stream")
            .await
            .map_err(AgentSessionError::Unavailable)?;
        Self::check(response)
    }

    /// Answer a `tool_call_pending` event the app ran itself.
    ///
    /// Tachyon can briefly report a pending call as missing (404), so the
    /// result alone is resent a few times; the tool itself is never rerun.
    pub async fn submit_tool_result(
        &self,
        session_id: &str,
        tool_id: &str,
        result: &str,
        is_error: bool,
    ) -> Result<(), AgentSessionError> {
        let path = format!("{}/agent/tool-result", session_path(session_id)?);
        let body = json!({"tool_id":tool_id,"result":result,"is_error":is_error});
        let mut attempt = 0;
        loop {
            let response = self
                .post(&path, &body, "application/json")
                .await
                .map_err(AgentSessionError::Unavailable)?;
            if response.status() == StatusCode::NOT_FOUND && attempt < TOOL_RESULT_RETRIES {
                tokio::time::sleep(self.tool_result_retry_base * (1 << attempt)).await;
                attempt += 1;
                continue;
            }
            return Self::check(response).map(|_| ());
        }
    }
}

/// A tool result as the model reads it: the JSON text, or a truncated preview
/// when it is longer than `budget` bytes.
pub fn tool_result_text(result: &Value, budget: usize) -> String {
    let text = result.to_string();
    if text.len() <= budget {
        return text;
    }
    let mut end = budget.saturating_sub(1000);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    json!({"truncated":true,"preview":&text[..end],"instruction":"Use pagination; this is incomplete."}).to_string()
}

/// One event from a server-sent event record (the text between blank
/// lines). `[DONE]` becomes `{"type":"done"}`; a `{type, data: {...}}`
/// envelope is unwrapped. Records without data or with invalid JSON give
/// `None`.
pub fn parse_sse_record(record: &str) -> Option<Value> {
    let data = record
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join("\n");
    if data.is_empty() {
        return None;
    }
    if data == "[DONE]" {
        return Some(json!({"type":"done"}));
    }
    let value: Value = serde_json::from_str(&data).ok()?;
    match (&value["data"], &value["type"]) {
        (Value::Object(inner), kind) => {
            let mut event = Value::Object(inner.clone());
            event["type"] = kind.clone();
            Some(event)
        }
        _ => Some(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(server: &MockServer) -> AgentSessionClient {
        AgentSessionClient::new(server.uri(), "token", "tn_op")
            .with_platform_id(Some("tn_platform".into()))
            .with_tool_result_retry_base(Duration::from_millis(1))
    }

    #[test]
    fn only_session_ids_reach_a_path() {
        assert_eq!(session_path("as_a-1").unwrap(), "/v1/llms/sessions/as_a-1");
        for bad in [
            "ch_1",
            "as_../x",
            "as_a/b",
            &format!("as_{}", "a".repeat(200)),
        ] {
            assert!(matches!(
                session_path(bad),
                Err(AgentSessionError::InvalidSession)
            ));
        }
    }

    #[tokio::test]
    async fn creates_a_session_with_the_callers_headers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/llms/sessions"))
            .and(header("authorization", "Bearer token"))
            .and(header("x-operator-id", "tn_op"))
            .and(header("x-platform-id", "tn_platform"))
            .and(body_json(json!({"name":"Agent"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"session_id":"as_1"})))
            .mount(&server)
            .await;
        let created = client(&server)
            .create_session(&json!({"name":"Agent"}))
            .await
            .unwrap();
        assert_eq!(created["session_id"], "as_1");
    }

    #[tokio::test]
    async fn rejects_a_session_response_without_a_valid_id() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"session_id":"x"})))
            .mount(&server)
            .await;
        assert!(matches!(
            client(&server).create_session(&json!({})).await,
            Err(AgentSessionError::InvalidResponse(_))
        ));
    }

    #[tokio::test]
    async fn reports_refusal_separately_from_failure() {
        let server = MockServer::start().await;
        Mock::given(path("/v1/llms/sessions/as_1/agent/execute"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&server)
            .await;
        Mock::given(path("/v1/llms/sessions/as_2/agent/execute"))
            .respond_with(ResponseTemplate::new(502))
            .mount(&server)
            .await;
        let client = client(&server);
        assert!(matches!(
            client.execute("as_1", &json!({})).await,
            Err(AgentSessionError::Unauthorized(StatusCode::FORBIDDEN))
        ));
        assert!(matches!(
            client.execute("as_2", &json!({})).await,
            Err(AgentSessionError::Status(StatusCode::BAD_GATEWAY))
        ));
    }

    #[tokio::test]
    async fn execute_passes_the_body_through_and_returns_the_stream() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/llms/sessions/as_1/agent/execute"))
            .and(header("accept", "text/event-stream"))
            .and(body_json(
                json!({"task":"x","attachments":[{"filename":"a.png"}]}),
            ))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string("data: [DONE]\n\n"),
            )
            .mount(&server)
            .await;
        let response = client(&server)
            .execute(
                "as_1",
                &json!({"task":"x","attachments":[{"filename":"a.png"}]}),
            )
            .await
            .unwrap();
        assert_eq!(response.text().await.unwrap(), "data: [DONE]\n\n");
    }

    #[tokio::test]
    async fn resends_a_tool_result_while_the_pending_call_is_missing() {
        let server = MockServer::start().await;
        Mock::given(path("/v1/llms/sessions/as_1/agent/tool-result"))
            .respond_with(ResponseTemplate::new(404))
            .up_to_n_times(2)
            .mount(&server)
            .await;
        Mock::given(path("/v1/llms/sessions/as_1/agent/tool-result"))
            .and(body_json(
                json!({"tool_id":"t1","result":"{}","is_error":false}),
            ))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;
        client(&server)
            .submit_tool_result("as_1", "t1", "{}", false)
            .await
            .unwrap();
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn stops_resending_after_the_retry_budget() {
        let server = MockServer::start().await;
        Mock::given(path("/v1/llms/sessions/as_1/agent/tool-result"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        assert!(matches!(
            client(&server)
                .submit_tool_result("as_1", "t1", "{}", false)
                .await,
            Err(AgentSessionError::Status(StatusCode::NOT_FOUND))
        ));
        assert_eq!(
            server.received_requests().await.unwrap().len(),
            1 + TOOL_RESULT_RETRIES as usize
        );
    }

    #[test]
    fn previews_large_tool_results_on_a_char_boundary() {
        assert_eq!(tool_result_text(&json!({"ok":true}), 100), r#"{"ok":true}"#);
        let text = tool_result_text(&json!("あ".repeat(2000)), 1500);
        let preview: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(preview["truncated"], true);
        assert!(preview["preview"].as_str().unwrap().len() <= 500);
    }

    #[test]
    fn parses_sse_records() {
        assert_eq!(
            parse_sse_record("data: [DONE]"),
            Some(json!({"type":"done"}))
        );
        assert_eq!(
            parse_sse_record("event: x\ndata: {\"type\":\"say\",\"data\":{\"text\":\"hi\"}}"),
            Some(json!({"type":"say","text":"hi"}))
        );
        assert_eq!(
            parse_sse_record("data: {\"type\":\"say\",\"text\":\"hi\"}"),
            Some(json!({"type":"say","text":"hi"}))
        );
        assert_eq!(parse_sse_record(": keep-alive"), None);
        assert_eq!(parse_sse_record("data: {oops"), None);
    }
}
