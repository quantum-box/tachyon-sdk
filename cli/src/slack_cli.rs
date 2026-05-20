//! `tachyon slack` subcommand for tenant-scoped Slack integrations.

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};

#[derive(Debug, Clone, Args)]
pub struct SlackArgs {
    #[command(subcommand)]
    pub command: SlackCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SlackCommand {
    /// Send a Slack message using a connected integration
    Send {
        /// Integration ID
        #[arg(long)]
        integration: String,
        /// Slack channel ID or name
        #[arg(long)]
        channel: String,
        /// Message text to send
        #[arg(long)]
        message: String,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct SendSlackMessageRequest {
    channel: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct SendSlackMessageResponse {
    ok: bool,
    channel: String,
    message_ts: String,
}

fn send_message_path(integration_id: &str) -> String {
    format!("/v1/integrations/{integration_id}/messages")
}

async fn run_send(
    api: &ApiClient,
    integration: &str,
    channel: &str,
    message: &str,
    json: bool,
) -> Result<()> {
    let request = SendSlackMessageRequest {
        channel: channel.to_string(),
        message: message.to_string(),
    };
    let response: SendSlackMessageResponse =
        api.post(&send_message_path(integration), &request).await?;

    if json {
        return print_json(&response);
    }

    println!(
        "Slack message sent: channel={}, ts={}",
        response.channel, response.message_ts
    );
    Ok(())
}

pub async fn run(args: &SlackArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        SlackCommand::Send {
            integration,
            channel,
            message,
            json,
        } => run_send(&api, integration, channel, message, *json).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_send_message_path() {
        assert_eq!(
            send_message_path("int_slack"),
            "/v1/integrations/int_slack/messages"
        );
    }

    #[test]
    fn serializes_send_message_request() {
        let request = SendSlackMessageRequest {
            channel: "#tachyon-test".to_string(),
            message: "hello".to_string(),
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            serde_json::json!({
                "channel": "#tachyon-test",
                "message": "hello"
            })
        );
    }
}
