use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use clap::{Args, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, Password};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tachyon_sdk::apis::configuration::Configuration;
use tokio::time::{sleep, Duration};

use crate::build_reproduce;
use crate::client::{print_json, truncate, ApiClient};
use crate::config::loader::ProjectConfig;
use crate::resolve;

const BUILD_LOGS_FOLLOW_INTERVAL: Duration = Duration::from_secs(2);
const BUILD_LOGS_MAX_NO_PROGRESS_NONE_TOKEN_POLLS: usize = 3;

#[derive(Debug, Clone, Args)]
pub struct ComputeArgs {
    #[command(subcommand)]
    pub command: ComputeCommand,
}

#[derive(Debug, Clone, Args)]
pub struct EnvArgs {
    #[command(subcommand)]
    pub command: EnvCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ComputeCommand {
    /// List compute apps
    Apps {
        #[command(subcommand)]
        command: AppsCommand,
    },
    /// Manage builds
    Builds {
        #[command(subcommand)]
        command: BuildsCommand,
    },
    /// Manage preview builds
    Preview(PreviewArgs),
    /// Manage deployments
    Deployments {
        #[command(subcommand)]
        command: DeploymentsCommand,
    },
    /// Manage environment variables
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },
    /// Manage custom domains
    Domains {
        #[command(subcommand)]
        command: DomainsCommand,
    },
    /// Manage scaling configuration
    Scaling {
        #[command(subcommand)]
        command: ScalingCommand,
    },
    /// Build a Cloudflare Pages app locally (emulates CodeBuild pipeline)
    Build {
        /// App to build (tachyon, cms, docs)
        app: PagesApp,
        /// Also deploy to Cloudflare Pages preview environment
        #[arg(long)]
        deploy: bool,
        /// Project root directory (defaults to current directory)
        #[arg(long)]
        project_dir: Option<PathBuf>,
    },
    /// Build and start local preview server (wrangler pages dev)
    Dev {
        /// App to preview (tachyon, cms, docs)
        app: PagesApp,
        /// Project root directory (defaults to current directory)
        #[arg(long)]
        project_dir: Option<PathBuf>,
        /// Port for the preview server
        #[arg(long, default_value_t = 8788)]
        port: u16,
    },
    /// Show build status for a compute app (shortcut for builds list)
    Status {
        /// App ID or name
        app_id: Option<String>,
        /// Maximum number of builds to display
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Stream or fetch build logs (shortcut for builds logs)
    Logs {
        /// App ID or name (optional when --build-id is specified)
        app_id: Option<String>,
        /// Tail runtime logs for a Cloudflare-backed app
        #[arg(long, value_name = "APP_ID")]
        tail: Option<String>,
        /// Build ID (defaults to the latest build for the given app)
        #[arg(long)]
        build_id: Option<String>,
        /// Keep polling until the build is complete;
        /// exits with code 1 if the build fails
        #[arg(long)]
        follow: bool,
        /// Emit compact JSON Lines for coding agents
        #[arg(long)]
        agent: bool,
        /// Emit raw runtime log JSON Lines when used with --tail
        #[arg(long)]
        json: bool,
    },
}

mod apps;
mod builds;
mod deployments;
mod domains;
mod env;
mod local;
mod scaling;

pub use apps::AppsCommand;
pub use builds::BuildsCommand;
pub use deployments::DeploymentsCommand;
pub use domains::DomainsCommand;
pub use env::EnvCommand;
pub use local::PagesApp;
pub use scaling::ScalingCommand;

pub(crate) use apps::{
    app_entry_to_api_body, normalize_cloud_apps_document, plan_env_vars, run_apps_apply,
    run_apps_apply_manifest, select_app_entries, AppsApplyManifestInput,
};
pub(crate) use env::validate_secret_key;

use apps::*;
use builds::*;
use deployments::*;
use domains::*;
use env::*;
use local::*;
use scaling::*;

// ---- Formatting helpers ----

fn format_timestamp_ms(millis: i64) -> String {
    match Utc.timestamp_millis_opt(millis) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => format!("{millis}"),
    }
}

fn format_created_at(created_at: &str) -> String {
    if let Ok(dt) = created_at.parse::<DateTime<Utc>>() {
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    created_at.to_string()
}

fn truncate_sha(sha: &str) -> &str {
    if sha.len() > 8 {
        &sha[..8]
    } else {
        sha
    }
}

fn is_terminal_build_status(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "succeeded" | "failed" | "cancelled" | "canceled" | "timed_out" | "timeout"
    )
}

fn is_success_build_status(status: &str) -> bool {
    matches!(status.to_ascii_lowercase().as_str(), "succeeded")
}

fn is_http_not_found_error(err: &anyhow::Error) -> bool {
    crate::client::http_error_status(err) == Some(reqwest::StatusCode::NOT_FOUND)
}

fn print_agent_event(event: &AgentBuildEvent<'_>) -> Result<()> {
    println!("{}", serde_json::to_string(event)?);
    Ok(())
}

fn compact_agent_message(message: &str) -> String {
    const MAX_LEN: usize = 500;
    if message.chars().count() <= MAX_LEN {
        return message.to_string();
    }
    let mut compacted: String = message.chars().take(MAX_LEN - 3).collect();
    compacted.push_str("...");
    compacted
}

// ---- Entry point ----

fn app_id_or_default<'a>(
    app_id: &'a Option<String>,
    project_config: Option<&'a ProjectConfig>,
) -> Result<&'a str> {
    app_id_or_default_value(app_id.as_deref(), project_config)
}

fn app_id_or_default_value<'a>(
    app_id: Option<&'a str>,
    project_config: Option<&'a ProjectConfig>,
) -> Result<&'a str> {
    app_id
        .or_else(|| {
            project_config
                .and_then(|config| config.metadata.name.as_deref())
                .filter(|name| !name.is_empty())
        })
        .ok_or_else(|| anyhow!("app_id is required (or set metadata.name in tachyon.yml)"))
}

pub async fn run(
    args: &ComputeArgs,
    config: &Configuration,
    tenant_id: &str,
    project_config: Option<&ProjectConfig>,
    config_flag: Option<&Path>,
) -> Result<()> {
    // Local-only commands (no API call needed)
    match &args.command {
        ComputeCommand::Build {
            app,
            deploy,
            project_dir,
        } => return run_local_build(app, project_dir.as_ref(), *deploy),
        ComputeCommand::Dev {
            app,
            project_dir,
            port,
        } => return run_local_dev(app, project_dir.as_ref(), *port),
        _ => {}
    }

    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        ComputeCommand::Build { .. } | ComputeCommand::Dev { .. } => {
            unreachable!()
        }
        ComputeCommand::Apps { command } => match command {
            AppsCommand::List { json } => run_apps_list(&api, *json).await,
            AppsCommand::Get { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_apps_get(&api, &id, *json).await
            }
            AppsCommand::Delete { app_id } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_apps_delete(&api, &id).await
            }
            AppsCommand::Apply {
                file,
                app,
                environment,
                change_control_token,
                dry_run,
            } => {
                run_apps_apply(
                    &api,
                    tenant_id,
                    file,
                    app.as_deref(),
                    environment,
                    change_control_token.as_deref(),
                    *dry_run,
                )
                .await
            }
            AppsCommand::SyncSecrets {
                file,
                app,
                environment,
                dry_run,
            } => run_apps_sync_secrets(&api, file, app.as_deref(), environment, *dry_run).await,
            AppsCommand::Feedback(feedback_args) => {
                let id = resolve::resolve_app_id(&api, &feedback_args.app_id).await?;
                run_apps_feedback(tenant_id, &id, feedback_args)
            }
        },
        ComputeCommand::Preview(preview_args) => match &preview_args.command {
            Some(PreviewCommand::Create { app, branch }) => {
                run_preview_create(&api, app, branch).await
            }
            None => {
                run_preview(
                    &api,
                    &preview_args.app_id,
                    project_config,
                    preview_args.branch.as_deref(),
                    preview_args.pr,
                )
                .await
            }
        },
        ComputeCommand::Builds { command } => match command {
            BuildsCommand::List {
                app_id,
                limit,
                json,
            } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_builds_list(&api, &id, *limit, *json).await
            }
            BuildsCommand::Get { build_id, json } => run_builds_get(&api, build_id, *json).await,
            BuildsCommand::Trigger { app_id, branch, pr } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_builds_trigger(&api, &id, branch.as_deref(), *pr).await
            }
            BuildsCommand::Cancel { build_id } => run_builds_cancel(&api, build_id).await,
            BuildsCommand::Logs {
                app_id,
                build_id,
                follow,
                agent,
            } => {
                let resolved_app_id = match app_id {
                    Some(id) => Some(resolve::resolve_app_id(&api, id).await?),
                    None if build_id.is_none() => {
                        let app_id = app_id_or_default(app_id, project_config)?;
                        Some(resolve::resolve_app_id(&api, app_id).await?)
                    }
                    None => None,
                };
                run_builds_logs(
                    &api,
                    resolved_app_id.as_deref(),
                    build_id.as_deref(),
                    *follow,
                    *agent,
                )
                .await
            }
            BuildsCommand::Watch {
                app_id,
                build_id,
                interval_secs,
                timeout_secs,
                no_logs,
                agent,
            } => {
                let resolved_app_id = match app_id {
                    Some(id) => Some(resolve::resolve_app_id(&api, id).await?),
                    None if build_id.is_none() => {
                        let app_id = app_id_or_default(app_id, project_config)?;
                        Some(resolve::resolve_app_id(&api, app_id).await?)
                    }
                    None => None,
                };
                run_builds_watch(
                    &api,
                    resolved_app_id.as_deref(),
                    build_id.as_deref(),
                    *interval_secs,
                    *timeout_secs,
                    *no_logs,
                    *agent,
                )
                .await
            }
            BuildsCommand::Reproduce {
                build_id,
                mock,
                source_dir,
                image,
                dry_run,
            } => run_builds_reproduce(
                build_id,
                mock.as_deref(),
                source_dir.as_deref(),
                image.as_deref(),
                *dry_run,
            ),
            BuildsCommand::RunJob { spec_env } => {
                crate::cloud_app_build_job::run_from_env(spec_env).await
            }
        },
        ComputeCommand::Deployments { command } => match command {
            DeploymentsCommand::List { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_deployments_list(&api, &id, *json).await
            }
            DeploymentsCommand::Get {
                deployment_id,
                json,
            } => run_deployments_get(&api, deployment_id, *json).await,
            DeploymentsCommand::Cancel { deployment_id } => {
                run_deployments_cancel(&api, deployment_id).await
            }
            DeploymentsCommand::Rollback {
                app_id,
                deployment_id,
            } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_deployments_rollback(&api, &id, deployment_id).await
            }
        },
        ComputeCommand::Env { command } => match command {
            EnvCommand::List { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_list(&api, &id, *json).await
            }
            EnvCommand::Set {
                app_id,
                app,
                secret,
                value,
                target,
                branch,
                vars,
            } => {
                let mut vars = vars.clone();
                let positional_app = app_id.as_ref().and_then(|value| {
                    if value.contains('=') {
                        vars.insert(0, value.clone());
                        None
                    } else {
                        Some(value)
                    }
                });
                if let Some(key) = secret {
                    let mut selected_app = app.as_ref().or(positional_app);
                    let mut key = key.as_str();
                    let extra_vars = if app.is_none()
                        && vars.is_empty()
                        && positional_app.is_some_and(|value| {
                            !looks_like_secret_key(key) && looks_like_secret_key(value)
                        }) {
                        selected_app = secret.as_ref();
                        key = positional_app.expect("checked above").as_str();
                        &[] as &[String]
                    } else if app.is_none()
                        && positional_app.is_none()
                        && vars.len() == 1
                        && !vars[0].contains('=')
                    {
                        selected_app = secret.as_ref();
                        key = vars[0].as_str();
                        &[] as &[String]
                    } else {
                        vars.as_slice()
                    };
                    if !extra_vars.is_empty() {
                        return Err(anyhow!("KEY=VALUE arguments cannot be used with --secret"));
                    }
                    let app_id =
                        app_id_or_default_value(selected_app.map(String::as_str), project_config)?;
                    let id = resolve::resolve_app_id(&api, app_id).await?;
                    run_env_set_secret(&api, &id, key, target, value.as_deref(), config_flag).await
                } else {
                    let selected_app = app.as_ref().or(positional_app);
                    let app_id =
                        app_id_or_default_value(selected_app.map(String::as_str), project_config)?;
                    let id = resolve::resolve_app_id(&api, app_id).await?;
                    run_env_set(&api, &id, &vars, target, branch.as_deref()).await
                }
            }
            EnvCommand::Unset { app, target, args } => {
                let (positional_app, key) = match args.as_slice() {
                    [key] => (None, key),
                    [app_id, key] => (Some(app_id), key),
                    _ => unreachable!("clap enforces one or two unset args"),
                };
                let selected_app = app.as_ref().or(positional_app);
                let app_id =
                    app_id_or_default_value(selected_app.map(String::as_str), project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_unset_key(&api, &id, key, target.as_deref()).await
            }
            EnvCommand::Delete { app_id, env_id } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_delete(&api, &id, env_id).await
            }
        },
        ComputeCommand::Domains { command } => match command {
            DomainsCommand::List { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_domains_list(&api, &id, *json).await
            }
            DomainsCommand::Add { app_id, domain } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_domains_add(&api, &id, domain).await
            }
            DomainsCommand::Verify { domain_id } => run_domains_verify(&api, domain_id).await,
            DomainsCommand::Remove { domain_id } => run_domains_remove(&api, domain_id).await,
        },
        ComputeCommand::Scaling { command } => match command {
            ScalingCommand::Get { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_scaling_get(&api, &id, *json).await
            }
            ScalingCommand::Update {
                app_id,
                min_instances,
                max_instances,
            } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_scaling_update(&api, &id, *min_instances, *max_instances).await
            }
        },
        // Legacy shortcuts
        ComputeCommand::Status { app_id, limit } => {
            let app_id = app_id_or_default(app_id, project_config)?;
            let id = resolve::resolve_app_id(&api, app_id).await?;
            run_builds_list(&api, &id, *limit, false).await
        }
        ComputeCommand::Logs {
            app_id,
            tail,
            build_id,
            follow,
            agent,
            json,
        } => {
            if let Some(app_id) = tail {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                return run_runtime_log_tail(&api, &id, RuntimeLogTailOptions { raw_json: *json })
                    .await;
            }
            if *json {
                return Err(anyhow!("--json is only supported with compute logs --tail"));
            }
            let resolved_app_id = match app_id {
                Some(id) => Some(resolve::resolve_app_id(&api, id).await?),
                None => None,
            };
            if resolved_app_id.is_none() && build_id.is_none() {
                return Err(anyhow!("either app_id or --build-id must be provided"));
            }
            run_builds_logs(
                &api,
                resolved_app_id.as_deref(),
                build_id.as_deref(),
                *follow,
                *agent,
            )
            .await
        }
    }
}

pub async fn run_env(
    args: &EnvArgs,
    config: &Configuration,
    tenant_id: &str,
    project_config: Option<&ProjectConfig>,
    config_flag: Option<&Path>,
) -> Result<()> {
    let compute_args = ComputeArgs {
        command: ComputeCommand::Env {
            command: args.command.clone(),
        },
    };
    run(
        &compute_args,
        config,
        tenant_id,
        project_config,
        config_flag,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn sentry_integration_adds_cli_plan_metadata() {
        let entry = json!({
            "name": "planet-library",
            "integrations": {
                "sentry": {
                    "project": "Next.js App"
                }
            },
            "envVars": [
                {
                    "name": "SENTRY_DSN",
                    "valueFrom": {
                        "secret": "SENTRY_DSN"
                    }
                }
            ]
        });

        let plan = plan_env_vars(&entry, "production").unwrap();

        assert_eq!(
            plan.secret_refs
                .iter()
                .map(|secret| secret.key.as_str())
                .collect::<Vec<_>>(),
            vec!["SENTRY_DSN"]
        );
        assert_eq!(
            plan.sentry_integrations,
            vec![SentryIntegrationPlan {
                project: "Next.js App".to_string(),
                provider: "sentry_next_js_app".to_string(),
                env_vars: vec!["NEXT_PUBLIC_SENTRY_DSN".to_string()],
            }]
        );
    }

    #[rstest]
    #[case(
        "DATABASE_URL",
        Some("production"),
        json!({
            "databaseRef": {
                "name": "tachyonfield",
                "field": "url"
            }
        }),
        "production",
        ServerManagedCredentialSource::Database
    )]
    #[case(
        "COGNITO_CLIENT_ID",
        None,
        json!({
            "oauth2ClientRef": {
                "name": "fieldadmin-web",
                "field": "clientId"
            }
        }),
        "production",
        ServerManagedCredentialSource::OAuth2Client
    )]
    #[case(
        "BLOB_BUCKET_NAME",
        Some("preview"),
        json!({
            "storageRef": {
                "name": "field-assets",
                "field": "bucketName"
            }
        }),
        "preview",
        ServerManagedCredentialSource::Storage
    )]
    fn server_managed_credential_ref_is_planned_without_secret_sync(
        #[case] key: &str,
        #[case] target: Option<&str>,
        #[case] value_from: Value,
        #[case] expected_target: &str,
        #[case] expected_source: ServerManagedCredentialSource,
    ) {
        let mut env_var = json!({
            "name": key,
            "type": "credential",
            "valueFrom": value_from
        });
        if let Some(target) = target {
            env_var["target"] = json!(target);
        }
        let entry = json!({
            "name": "tachyon-field-api",
            "envVars": [env_var]
        });

        let plan = plan_env_vars(&entry, "production").unwrap();

        assert!(plan.plain.is_empty());
        assert!(plan.secret_refs.is_empty());
        assert_eq!(
            plan.server_managed_credentials,
            vec![ServerManagedCredentialRef {
                key: key.to_string(),
                target: expected_target.to_string(),
                source: expected_source,
            }]
        );
    }

    #[test]
    fn cloud_app_manifest_for_iac_injects_tenant_and_removes_name() {
        let entry = json!({
            "name": "tachyon-field-api",
            "deploymentTarget": "lambda",
            "envVars": [
                {
                    "name": "DATABASE_URL",
                    "type": "credential",
                    "valueFrom": {
                        "databaseRef": {
                            "name": "tachyonfield",
                            "field": "url"
                        }
                    }
                }
            ]
        });

        let manifest = cloud_app_manifest_for_iac(&entry, "tn_01ks18jhh1xvggktfzjx5jqsen").unwrap();

        assert_eq!(manifest["kind"], "CloudApp");
        assert_eq!(manifest["metadata"]["name"], "tachyon-field-api");
        assert_eq!(
            manifest["metadata"]["tenantId"],
            "tn_01ks18jhh1xvggktfzjx5jqsen"
        );
        assert!(manifest["spec"].get("name").is_none());
        assert_eq!(manifest["spec"]["deploymentTarget"], "lambda");
    }

    #[test]
    fn sync_secrets_request_enumerates_refs_in_stable_order() {
        let entry = json!({
            "name": "fieldadmin",
            "envVars": [
                {
                    "name": "COGNITO_CLIENT_SECRET",
                    "type": "credential",
                    "target": "production",
                    "valueFrom": {
                        "oauth2ClientRef": {
                            "name": "fieldadmin-login",
                            "field": "clientSecret"
                        }
                    }
                },
                {
                    "name": "RESEND_API_KEY",
                    "type": "credential",
                    "valueFrom": {
                        "secret": "prod/resend/api-key"
                    }
                },
                {
                    "name": "DATABASE_URL",
                    "type": "credential",
                    "target": "preview",
                    "valueFrom": {
                        "databaseRef": {
                            "name": "fieldadmin",
                            "field": "url"
                        }
                    }
                },
                {
                    "name": "PUBLIC_BASE_URL",
                    "value": "https://fieldadmin.example"
                }
            ]
        });

        let request = build_sync_secrets_request(&entry, "production").unwrap();

        assert_eq!(request.app_name, "fieldadmin");
        assert_eq!(request.environment, "production");
        assert_eq!(request.refs.len(), 3);
        assert_eq!(
            request
                .refs
                .iter()
                .map(|reference| (
                    reference.key.as_str(),
                    reference.target.as_str(),
                    reference.source.as_str()
                ))
                .collect::<Vec<_>>(),
            vec![
                ("COGNITO_CLIENT_SECRET", "production", "oauth2ClientRef"),
                ("DATABASE_URL", "preview", "databaseRef"),
                ("RESEND_API_KEY", "production", "secretRef"),
            ]
        );
    }

    #[test]
    fn sync_secrets_rendering_omits_source_ref_details() {
        let refs = vec![
            SyncSecretRef {
                key: "COGNITO_CLIENT_SECRET".to_string(),
                target: "production".to_string(),
                source: "oauth2ClientRef".to_string(),
                source_ref: json!({
                    "name": "fieldadmin-login",
                    "field": "clientSecret"
                }),
            },
            SyncSecretRef {
                key: "RESEND_API_KEY".to_string(),
                target: "production".to_string(),
                source: "secretRef".to_string(),
                source_ref: json!("prod/resend/api-key"),
            },
        ];

        let rendered = render_sync_secret_refs(&refs).join(", ");

        assert_eq!(
            rendered,
            "COGNITO_CLIENT_SECRET(production; oauth2ClientRef), RESEND_API_KEY(production; secretRef)"
        );
        assert!(!rendered.contains("fieldadmin-login"));
        assert!(!rendered.contains("prod/resend/api-key"));
    }

    #[test]
    fn sync_secrets_requires_disambiguated_multi_app_manifest() {
        let manifest = json!({
            "kind": "CloudApps",
            "spec": {
                "apps": [
                    {"name": "fieldadmin"},
                    {"name": "fieldadmin-api"}
                ]
            }
        });

        let error = select_single_app_entry(&manifest, None)
            .unwrap_err()
            .to_string();

        assert!(error.contains("requires --app"));
    }

    #[test]
    fn runtime_tail_formatter_summarizes_cloudflare_request_logs_and_exceptions() {
        let raw = r#"{
            "eventTimestamp": 1767273332000,
            "scriptName": "tachyon-console",
            "cpuTime": 86,
            "request": {
                "method": "GET",
                "url": "https://example.com/api/reservations?limit=1"
            },
            "response": { "status": 200 },
            "logs": [
                { "level": "log", "message": ["reservation", 42] }
            ],
            "exceptions": [
                { "name": "TypeError", "message": "Cannot read property 'id' of undefined" }
            ]
        }"#;

        let lines = format_runtime_log_lines(raw);

        assert_eq!(
            lines,
            vec![
                "[13:15:32] GET /api/reservations?limit=1 200  (cpu: 86ms)  source=tachyon-console",
                "[13:15:32] ERROR TypeError: Cannot read property 'id' of undefined",
                "[13:15:32] LOG reservation 42",
            ]
        );
    }

    #[test]
    fn sentry_provider_name_is_stable() {
        assert_eq!(sentry_provider_name("nextjs"), "sentry_nextjs");
        assert_eq!(
            sentry_provider_name("Field Admin UI"),
            "sentry_field_admin_ui"
        );
    }

    #[test]
    fn runtime_tail_formatter_falls_back_for_unknown_payloads() {
        let json_lines = format_runtime_log_lines(r#"{"unexpected":{"nested":true}}"#);
        assert_eq!(json_lines.len(), 1);
        assert!(json_lines[0].starts_with('['));
        assert!(json_lines[0].contains(r#""unexpected":{"nested":true}"#));

        assert_eq!(
            format_runtime_log_lines("not json"),
            vec!["not json".to_string()]
        );
    }

    #[test]
    fn runtime_tail_formatter_includes_top_level_level_message_and_source() {
        let lines = format_runtime_log_lines(
            r#"{"timestamp":"2026-01-01T20:15:40Z","level":"error","message":"boom","source":"worker"}"#,
        );

        assert_eq!(lines, vec!["[20:15:40] ERROR boom source=worker"]);
    }

    #[test]
    fn runtime_tail_sse_parser_handles_multiline_data_and_crlf() {
        let mut pending =
            "event: log\r\ndata: {\"message\":\"a\"}\r\ndata: {\"message\":\"b\"}\r\n\r\n"
                .to_string();

        let events = drain_sse_events(&mut pending);

        assert!(pending.is_empty());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("log"));
        assert_eq!(
            events[0].data,
            vec![
                "{\"message\":\"a\"}".to_string(),
                "{\"message\":\"b\"}".to_string(),
            ]
        );
    }

    #[test]
    fn runtime_tail_utf8_buffer_preserves_multibyte_split_across_chunks() {
        let event = "event: log\ndata: {\"message\":\"日本語ログ\"}\n\n";
        let bytes = event.as_bytes();
        let split = event.find("語").unwrap() + "語".len() - 1;
        assert!(
            std::str::from_utf8(&bytes[..split]).is_err(),
            "test split must cut inside a multibyte codepoint"
        );

        let mut utf8 = Utf8ChunkBuffer::default();
        let mut pending = String::new();
        pending.push_str(&utf8.push_chunk(&bytes[..split]).unwrap());
        assert!(drain_sse_events(&mut pending).is_empty());

        pending.push_str(&utf8.push_chunk(&bytes[split..]).unwrap());
        pending.push_str(&utf8.finish().unwrap());
        let events = drain_sse_events(&mut pending);

        assert!(pending.is_empty());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("log"));
        assert_eq!(
            events[0].data,
            vec!["{\"message\":\"日本語ログ\"}".to_string()]
        );
    }

    #[test]
    fn feedback_payload_includes_cloud_app_context() {
        let args = FeedbackArgs {
            app_id: "app_01test".to_string(),
            message: "The production page returns 500.".to_string(),
            kind: FeedbackKind::Bug,
            severity: FeedbackSeverity::High,
            url: Some("https://example.txcloud.app".to_string()),
            build_id: Some("bld_01test".to_string()),
            deployment_id: Some("dep_01test".to_string()),
            contact: Some("user@example.com".to_string()),
            metadata: vec!["browser=Chrome".to_string()],
            json: false,
        };

        let payload = build_feedback_payload("tn_01test", "app_01resolved", &args).unwrap();

        assert_eq!(payload.app_id, "app_01resolved");
        assert_eq!(payload.operator_id, "tn_01test");
        assert_eq!(payload.kind, FeedbackKind::Bug);
        assert_eq!(payload.severity, FeedbackSeverity::High);
        assert_eq!(
            payload.metadata.get("browser").map(String::as_str),
            Some("Chrome")
        );
    }

    #[test]
    fn parse_github_remote_supports_ssh_and_https() {
        assert_eq!(
            parse_github_remote("git@github.com:quantum-box/moverent.git"),
            Some(("quantum-box".to_string(), "moverent".to_string()))
        );
        assert_eq!(
            parse_github_remote("https://github.com/quantum-box/moverent"),
            Some(("quantum-box".to_string(), "moverent".to_string()))
        );
        assert_eq!(
            parse_github_remote("ssh://git@github.com/quantum-box/moverent.git"),
            Some(("quantum-box".to_string(), "moverent".to_string()))
        );
    }

    #[test]
    fn feedback_metadata_rejects_secret_like_keys() {
        let err = parse_feedback_metadata(&["api_key=secret".to_string()]).unwrap_err();

        assert!(err.to_string().contains("secret-like"), "{err}");
    }

    #[test]
    fn feedback_markdown_formats_context() {
        let payload = FeedbackPayload {
            app_id: "app_01test".to_string(),
            operator_id: "tn_01test".to_string(),
            kind: FeedbackKind::Feature,
            severity: FeedbackSeverity::Medium,
            message: "Please add CSV export.".to_string(),
            url: None,
            build_id: None,
            deployment_id: None,
            contact: None,
            metadata: BTreeMap::from([("browser".to_string(), "Safari".to_string())]),
            created_at: "2026-05-29T00:00:00+00:00".to_string(),
        };

        let markdown = render_feedback_markdown(&payload);

        assert!(markdown.contains("# Cloud App Feedback"));
        assert!(markdown.contains("- App ID: app_01test"));
        assert!(markdown.contains("- Kind: feature"));
        assert!(markdown.contains("Please add CSV export."));
        assert!(markdown.contains("  - browser: Safari"));
    }

    #[test]
    fn parse_github_remote_rejects_non_github_urls() {
        assert_eq!(parse_github_remote("https://gitlab.com/a/b.git"), None);
        assert_eq!(parse_github_remote("git@github.com:invalid"), None);
    }
}
