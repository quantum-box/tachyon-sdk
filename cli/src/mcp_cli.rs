//! `tachyon mcp` subcommand: stdio / streamable-HTTP MCP server.

use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use rmcp::{
    transport::{
        stdio,
        streamable_http_server::{
            session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
        },
    },
    ServiceExt,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::mcp::{
    auth::{auth_layer, AuthConfig},
    openai::OpenAiClient,
    server::TachyonMcpServer,
};

#[derive(Debug, Clone, Args)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum McpCommand {
    /// Start the Tachyon MCP server.
    Serve {
        /// Transport: `stdio` (default, for Claude Code/Cursor local) or `http`
        /// (Streamable HTTP, MCP spec 2025-06-18).
        #[arg(long, value_enum, default_value_t = Transport::Stdio)]
        transport: Transport,

        /// Bind address for HTTP transport (e.g. `127.0.0.1:7337`).
        #[arg(long, default_value = "127.0.0.1:7337", env = "TACHYON_MCP_BIND")]
        bind: String,

        /// HTTP path for the MCP endpoint.
        #[arg(long, default_value = "/mcp")]
        path: String,

        /// Comma-separated list of accepted bearer tokens (HTTP transport only).
        /// If empty, no auth is required (local dev). Use `Authorization: Bearer <token>`.
        #[arg(long, env = "TACHYON_MCP_TOKENS", value_delimiter = ',')]
        tokens: Vec<String>,

        /// **Non-spec extension**: also accept `?apikey=<token>` URL-query auth.
        /// MCP spec 2025-06-18 §Access Token Usage forbids tokens in the URI.
        /// Enable only for tachyon-issued one-shot URLs; pair with short expiry
        /// and `Referrer-Policy: no-referrer` (set automatically).
        #[arg(long, default_value_t = false)]
        custom_query_auth: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Transport {
    Stdio,
    Http,
}

pub async fn run(args: &McpArgs) -> Result<()> {
    match &args.command {
        McpCommand::Serve {
            transport,
            bind,
            path,
            tokens,
            custom_query_auth,
        } => match transport {
            Transport::Stdio => serve_stdio().await,
            Transport::Http => serve_http(bind, path, tokens.clone(), *custom_query_auth).await,
        },
    }
}

fn init_tracing(stdio: bool) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // For stdio transport, all logs MUST go to stderr (stdout = MCP frames).
    let layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);
    let registry = tracing_subscriber::registry().with(filter).with(layer);
    let _ = registry.try_init();
    let _ = stdio; // currently both modes log to stderr
}

fn build_server() -> TachyonMcpServer {
    TachyonMcpServer::new(OpenAiClient::from_env())
}

fn warn_if_openai_missing() {
    if OpenAiClient::from_env().is_none() {
        eprintln!(
            "[tachyon mcp] OPENAI_API_KEY not set — \
             skipping `generate_image` tool registration"
        );
    }
}

async fn serve_stdio() -> Result<()> {
    init_tracing(true);
    eprintln!("[tachyon mcp] starting stdio transport");
    warn_if_openai_missing();
    let server = build_server();
    let service = server
        .serve(stdio())
        .await
        .context("MCP stdio handshake failed")?;
    service.waiting().await.context("MCP service ended")?;
    Ok(())
}

async fn serve_http(
    bind: &str,
    path: &str,
    tokens: Vec<String>,
    custom_query_auth: bool,
) -> Result<()> {
    init_tracing(false);
    warn_if_openai_missing();

    let auth_cfg = Arc::new(AuthConfig {
        tokens,
        allow_query_auth: custom_query_auth,
    });

    if !auth_cfg.is_enforced() {
        eprintln!(
            "[tachyon mcp] WARNING: no --tokens configured — HTTP transport will accept \
             unauthenticated requests. Suitable only for local dev."
        );
    }
    if custom_query_auth {
        eprintln!(
            "[tachyon mcp] WARNING: --custom-query-auth enabled. ?apikey=... is a \
             non-spec extension (MCP spec 2025-06-18 forbids tokens in URI). \
             Use short-lived tokens; Referrer-Policy: no-referrer is set on responses."
        );
    }

    let ct = tokio_util::sync::CancellationToken::new();
    let mcp_service = StreamableHttpService::new(
        || Ok(build_server()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
    );

    let auth_state = auth_cfg.clone();
    let app = axum::Router::new()
        .nest_service(path, mcp_service)
        .layer(axum::middleware::from_fn_with_state(auth_state, auth_layer));

    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("Failed to bind {bind}"))?;
    let local = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| bind.to_string());
    eprintln!("[tachyon mcp] listening on http://{local}{path} (Streamable HTTP)");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            ct.cancel();
        })
        .await
        .context("HTTP server failed")?;
    Ok(())
}
