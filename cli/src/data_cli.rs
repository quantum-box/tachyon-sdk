//! `tachyon data` subcommand — Tachyon Data datasets and queries.
//!
//! Usage:
//!   tachyon data dataset list
//!   tachyon data dataset create --name sales --file sales.csv \
//!       --purpose tenant_analytics
//!   tachyon data dataset get <dataset_id>
//!   tachyon data query run --sql "SELECT ..." --bind sales=<dataset_id> --wait
//!   tachyon data query get <job_id> [--wait]
//!
//! The commands talk to the Data surface that `tachyon-api` nests under
//! `/data`, so they reuse the profile credentials `tachyon auth login`
//! already stores. Nothing here prints, logs, or writes a token.
//!
//! Two shapes of the Data API need care:
//!
//! 1. A query binds an alias to a *dataset version*, not to a dataset. The
//!    `--bind name=<dataset_id>` form people want is resolved here by
//!    describing the dataset first and taking its published version, so the
//!    version that was read is always known and reportable.
//! 2. `POST /data/v1/query-jobs` answers `202` even when the backend refuses
//!    the query: the refusal arrives as `state: "failed"` with a
//!    `failure.code` such as `QUERY_TOO_EXPENSIVE`. Checking the HTTP status
//!    alone would report a refused query as accepted, so every job body is
//!    inspected.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand, ValueEnum};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{ApiClient, HttpError, RequestBody};

/// Path prefix `tachyon-api` nests the Data router under.
const DATA_PREFIX: &str = "/data/v1";
/// Header that declares why data is being read or written.
const PURPOSE_HEADER: &str = "x-data-purpose";
/// Upload cap enforced by `POST /data/v1/datasets` (64 MiB).
const MAX_UPLOAD_BYTES: u64 = 64 * 1024 * 1024;
/// Default seconds between `--wait` polls of a query job.
const DEFAULT_POLL_INTERVAL_SECS: u64 = 2;
/// Default seconds `--wait` keeps polling before giving up. The server caps
/// execution well below this, so hitting it means the job is stuck rather
/// than slow.
const DEFAULT_WAIT_TIMEOUT_SECS: u64 = 300;
#[derive(Debug, Clone, Args)]
pub struct DataArgs {
    #[command(subcommand)]
    pub command: DataCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DataCommand {
    /// Manage Tachyon Data datasets
    Dataset(DatasetArgs),
    /// Run SQL against Tachyon Data datasets
    Query(QueryArgs),
}

#[derive(Debug, Clone, Args)]
pub struct DatasetArgs {
    #[command(subcommand)]
    pub command: DatasetCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DatasetCommand {
    /// List the tenant's datasets, most recently updated first
    List {
        /// Maximum datasets to return (server clamps to 1..=200)
        #[arg(long)]
        limit: Option<u32>,
        /// Datasets to skip before the first returned row
        #[arg(long)]
        offset: Option<u32>,
        /// Print the result as JSON instead of a table
        #[arg(long)]
        json: bool,
    },

    /// Upload a CSV and publish it as a new dataset version
    ///
    /// An existing name adds a version to that dataset rather than creating
    /// a second one.
    Create {
        /// Dataset name
        #[arg(long)]
        name: String,
        /// CSV file to upload. The first row is read as the header.
        #[arg(long)]
        file: PathBuf,
        /// Human-readable description stored with the dataset
        #[arg(long)]
        description: Option<String>,
        /// Why the data is being uploaded. Omitted means `unspecified`.
        #[arg(long, value_enum)]
        purpose: Option<Purpose>,
        /// Print the result as JSON instead of text
        #[arg(long)]
        json: bool,
    },

    /// Describe a dataset: its schema, published version, and SQL dialect
    Get {
        /// Dataset ID
        dataset_id: String,
        /// Pin a specific version instead of the latest published one
        #[arg(long)]
        version: Option<String>,
        /// Why the data is being read. Omitted means `unspecified`.
        #[arg(long, value_enum)]
        purpose: Option<Purpose>,
        /// Print the result as JSON instead of text
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Args)]
pub struct QueryArgs {
    #[command(subcommand)]
    pub command: QueryCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum QueryCommand {
    /// Submit a query job
    Run {
        /// SQL to run. It may only name the aliases it binds.
        #[arg(long)]
        sql: String,
        /// Bind a SQL alias to a dataset: `name=<dataset_id>`,
        /// `name=<dataset_id>@<version_id>` to pin a version, or
        /// `name=<version_id>` to bind one directly. Repeatable.
        #[arg(long = "bind", value_name = "NAME=DATASET_ID[@VERSION]")]
        binds: Vec<String>,
        /// Why the data is being read. Omitted means `unspecified`.
        #[arg(long, value_enum)]
        purpose: Option<Purpose>,
        /// Poll until the job finishes, then print the result
        #[arg(long)]
        wait: bool,
        /// Seconds between polls while waiting
        #[arg(long, default_value_t = DEFAULT_POLL_INTERVAL_SECS)]
        interval_secs: u64,
        /// Seconds to keep waiting before giving up
        #[arg(long, default_value_t = DEFAULT_WAIT_TIMEOUT_SECS)]
        timeout_secs: u64,
        /// Maximum rows to read back (server default 1000, clamped to 1..=10000)
        #[arg(long)]
        max_rows: Option<u32>,
        /// Print the result as JSON instead of a table
        #[arg(long)]
        json: bool,
    },

    /// Show a query job, optionally waiting for it to finish
    Get {
        /// Query job ID
        job_id: String,
        /// Poll until the job finishes, then print the result
        #[arg(long)]
        wait: bool,
        /// Seconds between polls while waiting
        #[arg(long, default_value_t = DEFAULT_POLL_INTERVAL_SECS)]
        interval_secs: u64,
        /// Seconds to keep waiting before giving up
        #[arg(long, default_value_t = DEFAULT_WAIT_TIMEOUT_SECS)]
        timeout_secs: u64,
        /// Maximum rows to read back (server default 1000, clamped to 1..=10000)
        #[arg(long)]
        max_rows: Option<u32>,
        /// Print the result as JSON instead of a table
        #[arg(long)]
        json: bool,
    },
}

/// Declarable values of the `x-data-purpose` header.
///
/// `unspecified` is what the server assumes when the header is absent, so it
/// is not offered as a choice: omitting `--purpose` says the same thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum Purpose {
    TenantOperations,
    TenantAnalytics,
    AgentAssist,
    VendorOperations,
    VendorAnalytics,
}

impl Purpose {
    fn as_str(self) -> &'static str {
        match self {
            Self::TenantOperations => "tenant_operations",
            Self::TenantAnalytics => "tenant_analytics",
            Self::AgentAssist => "agent_assist",
            Self::VendorOperations => "vendor_operations",
            Self::VendorAnalytics => "vendor_analytics",
        }
    }
}

// ---------------------------------------------------------------- wire types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSummary {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Absent while no version of the dataset has reached `READY`, which
    /// means the dataset is listed but cannot be queried yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetPage {
    pub items: Vec<DatasetSummary>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub r#type: String,
    pub nullable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSchema {
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlDialect {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveAccess {
    pub purpose: String,
    #[serde(default)]
    pub withheld_columns: Vec<String>,
    pub row_filtered: bool,
    pub access_revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetDescription {
    pub dataset: DatasetSummary,
    pub version: String,
    pub schema: DatasetSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sql_dialect: Option<SqlDialect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access: Option<EffectiveAccess>,
    /// Present only for datasets fed by a first-party source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingestion: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedVersion {
    pub dataset_id: String,
    pub version_id: String,
    pub row_count: u64,
}

#[derive(Debug, Clone, Serialize)]
struct QueryJobSpec {
    sql: String,
    bindings: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_rows: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryJobFailure {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryJobStatus {
    pub job_id: String,
    pub state: String,
    #[serde(default)]
    pub bindings: BTreeMap<String, String>,
    #[serde(default)]
    pub read_versions: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sql_dialect: Option<SqlDialect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bytes_billed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_billed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_count: Option<u64>,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<QueryJobFailure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_available_until: Option<DateTime<Utc>>,
    pub submitted_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_requested_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
    #[serde(default = "unspecified_purpose")]
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_revision: Option<String>,
}

fn unspecified_purpose() -> String {
    "unspecified".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultPage {
    pub job_id: String,
    pub schema: DatasetSchema,
    pub rows: Vec<Vec<Value>>,
    pub total_rows: u64,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub read_versions: BTreeMap<String, String>,
}

/// Error envelope shared by every Tachyon REST endpoint.
#[derive(Debug, Clone, Deserialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

// ------------------------------------------------------------ job state help

/// Job states that will not change again.
fn is_terminal(state: &str) -> bool {
    matches!(state, "succeeded" | "failed" | "cancelled")
}

// -------------------------------------------------------------- bind parsing

/// One `--bind` argument, before the dataset is resolved to a version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindSpec {
    pub alias: String,
    pub target: BindTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindTarget {
    /// Describe this dataset and read the version it resolves to.
    Dataset {
        dataset_id: String,
        version: Option<String>,
    },
    /// A version the caller named directly; nothing to resolve.
    Version(String),
}

/// Parse `name=<dataset_id>[@<version_id>]`.
///
/// A value that looks like a version ID is taken as one, so the
/// `version_id` printed by `dataset create` can be pasted straight back in.
pub fn parse_bind(arg: &str) -> Result<BindSpec> {
    let (alias, value) = arg.split_once('=').ok_or_else(|| {
        anyhow!("--bind expects NAME=DATASET_ID[@VERSION], got `{arg}` with no `=`")
    })?;
    let alias = alias.trim();
    let value = value.trim();
    if alias.is_empty() {
        return Err(anyhow!("--bind `{arg}` has an empty alias"));
    }
    if value.is_empty() {
        return Err(anyhow!("--bind `{arg}` has an empty dataset ID"));
    }

    let target = match value.split_once('@') {
        Some((dataset_id, version)) => {
            let dataset_id = dataset_id.trim();
            let version = version.trim();
            if dataset_id.is_empty() {
                return Err(anyhow!("--bind `{arg}` has an empty dataset ID"));
            }
            if version.is_empty() {
                return Err(anyhow!("--bind `{arg}` has an empty version ID"));
            }
            BindTarget::Dataset {
                dataset_id: dataset_id.to_string(),
                version: Some(version.to_string()),
            }
        }
        None if value.starts_with("dsv_") => BindTarget::Version(value.to_string()),
        None => BindTarget::Dataset {
            dataset_id: value.to_string(),
            version: None,
        },
    };

    Ok(BindSpec {
        alias: alias.to_string(),
        target,
    })
}

/// Parse every `--bind`, rejecting a repeated alias rather than letting the
/// last one silently win.
pub fn parse_binds(args: &[String]) -> Result<Vec<BindSpec>> {
    let mut seen: Vec<String> = Vec::new();
    let mut specs = Vec::with_capacity(args.len());
    for arg in args {
        let spec = parse_bind(arg)?;
        if seen.contains(&spec.alias) {
            return Err(anyhow!(
                "--bind names the alias `{}` more than once",
                spec.alias
            ));
        }
        seen.push(spec.alias.clone());
        specs.push(spec);
    }
    Ok(specs)
}

// ------------------------------------------------------------ error messages

/// Rewrite a Data API failure into something a CLI user can act on.
///
/// The gate that `tachyon-api` puts in front of `/data` answers `404` with an
/// empty body when the tenant does not have the `data_platform` flag, which
/// otherwise reads as "your dataset is missing".
pub fn explain_http_error(err: anyhow::Error, tenant_id: &str) -> anyhow::Error {
    let Some(http) = err.downcast_ref::<HttpError>() else {
        return err;
    };
    let status = http.status;
    let envelope: Option<ErrorResponse> = serde_json::from_str(&http.body).ok();

    let detail = match &envelope {
        Some(e) => format!("{}: {}", e.code, e.message),
        None if http.body.trim().is_empty() => String::new(),
        None => http.body.trim().to_string(),
    };

    let hint = match status.as_u16() {
        404 if envelope.is_none() && http.body.trim().is_empty() => format!(
            "Tachyon Data is not enabled for tenant {tenant_id}. `/data/v1/*` is gated by \
             the `data_platform` feature flag, and a tenant without it gets 404 before \
             authentication. Ask an operator to enable the flag, or check that \
             --api-url points at a tachyon-api that serves Tachyon Data."
        ),
        404 => format!("Not found on tenant {tenant_id}. {detail}"),
        403 => format!(
            "Forbidden on tenant {tenant_id}. The signed-in user needs the `data:*` action \
             for this call (AdministratorAccess covers all of them). {detail}"
        ),
        401 => return err,
        429 => {
            let wait = http
                .retry_after_secs
                .map(|secs| format!(" Retry after {secs}s."))
                .unwrap_or_default();
            format!("Rate limited by Tachyon Data.{wait} {detail}")
        }
        503 => format!("Tachyon Data is temporarily unavailable. {detail}"),
        _ => return err,
    };

    anyhow!("{} {} failed: {hint}", http.method, http.path)
}

/// Explain a `failure.code` that a query job carries.
///
/// These arrive inside a `202` body, so they are not HTTP errors and never
/// pass through [`explain_http_error`].
pub fn explain_failure(failure: &QueryJobFailure) -> String {
    let hint = match failure.code.as_str() {
        "QUERY_TOO_EXPENSIVE" => Some(
            "The query exceeded the per-query limit: the scan budget on a metered backend \
             (BigQuery, 10 GiB by default), or execution time and memory on TiDB (30s and \
             1 GiB by default). Narrow the scan with a filter or fewer columns.",
        ),
        "TENANT_BUDGET_EXHAUSTED" => Some(
            "The tenant's monthly scan budget (100 GiB by default) has too little left for \
             this query. Wait for the budget to roll over or ask an operator to raise it.",
        ),
        "TENANT_QUERY_CONCURRENCY" => Some(
            "The tenant already has the maximum number of query jobs open (16 by default). \
             Wait about 30s for one to finish and submit again.",
        ),
        "TIMEOUT" => {
            Some("The backend stopped the query at its execution-time limit (30s by default).")
        }
        "INVALID_QUERY" => Some(
            "The backend rejected the SQL. A query may only name the aliases it binds, so \
             every table in the SQL needs a matching --bind.",
        ),
        "BACKEND_UNAVAILABLE" => {
            Some("The analytical backend did not answer. The query can be submitted again.")
        }
        "DATASET_VERSION_RETIRED" => Some(
            "A bound dataset version has been deleted or aged out. Re-resolve the binding \
             against the dataset's current version.",
        ),
        _ => None,
    };

    match hint {
        Some(hint) => format!("{}: {}\n{hint}", failure.code, failure.message),
        None => format!("{}: {}", failure.code, failure.message),
    }
}

// ----------------------------------------------------------------- rendering

/// Render one result cell. Strings print bare so a table of text stays
/// readable; everything else prints as the JSON it is.
fn render_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Print a left-aligned table. Widths are counted in characters, which is
/// right for ASCII IDs and close enough for the rest.
fn print_table(headers: &[String], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.chars().count());
            }
        }
    }

    let line = |cells: &[String]| {
        let rendered: Vec<String> = cells
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let width = widths.get(i).copied().unwrap_or(0);
                let pad = width.saturating_sub(cell.chars().count());
                format!("{cell}{}", " ".repeat(pad))
            })
            .collect();
        println!("{}", rendered.join("  ").trim_end());
    };

    line(headers);
    let separator: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    line(&separator);
    for row in rows {
        line(row);
    }
}

/// The provenance of a finished query, assembled from the job record.
///
/// The REST surface does not return a `provenance` object — the agent tool
/// does. This mirrors those fields so a CLI result can be cited the same way.
#[derive(Debug, Clone, Serialize)]
pub struct Provenance {
    pub query_job_id: String,
    pub tenant_id: String,
    pub bindings: BTreeMap<String, String>,
    pub read_versions: BTreeMap<String, String>,
    /// Absent when the SQL is not known to this invocation, which is the
    /// case for `query get` on a job someone else submitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_dialect: Option<SqlDialect>,
    pub purpose: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_revision: Option<String>,
    pub submitted_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_billed: Option<u64>,
}

/// SHA-256 of the SQL, so a result can be tied to the exact text that ran.
pub fn sql_sha256(sql: &str) -> String {
    let digest = Sha256::digest(sql.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl Provenance {
    fn build(job: &QueryJobStatus, tenant_id: &str, sql: Option<&str>) -> Self {
        Self {
            query_job_id: job.job_id.clone(),
            tenant_id: tenant_id.to_string(),
            bindings: job.bindings.clone(),
            read_versions: job.read_versions.clone(),
            sql_sha256: sql.map(sql_sha256),
            sql_dialect: job.sql_dialect.clone(),
            purpose: job.purpose.clone(),
            access_revision: job.access_revision.clone(),
            submitted_at: job.submitted_at,
            finished_at: job.finished_at,
            estimated_bytes: job.estimated_bytes,
            bytes_billed: job.bytes_billed,
        }
    }

    fn print(&self) {
        println!("Provenance:");
        println!("  query job:       {}", self.query_job_id);
        println!("  tenant:          {}", self.tenant_id);
        if let Some(digest) = &self.sql_sha256 {
            println!("  sql sha256:      {digest}");
        }
        if let Some(dialect) = &self.sql_dialect {
            println!("  sql dialect:     {} {}", dialect.name, dialect.version);
        }
        println!("  purpose:         {}", self.purpose);
        if let Some(revision) = &self.access_revision {
            println!("  access revision: {revision}");
        }
        if self.read_versions.is_empty() {
            println!("  read versions:   (none)");
        } else {
            println!("  read versions:");
            for (alias, version) in &self.read_versions {
                println!("    {alias} = {version}");
            }
        }
        println!("  submitted at:    {}", self.submitted_at);
        if let Some(finished) = self.finished_at {
            println!("  finished at:     {finished}");
        }
        if let Some(bytes) = self.estimated_bytes {
            println!("  estimated bytes: {bytes}");
        }
        if let Some(bytes) = self.bytes_billed {
            println!("  bytes billed:    {bytes}");
        }
    }
}

// -------------------------------------------------------------------- client

/// Thin wrapper that adds the `/data/v1` prefix, the optional purpose header,
/// and Data-specific error messages.
struct DataApi<'a> {
    api: ApiClient,
    tenant_id: &'a str,
}

impl<'a> DataApi<'a> {
    fn new(config: &Configuration, tenant_id: &'a str) -> Result<Self> {
        Ok(Self {
            api: ApiClient::new(config, tenant_id)?,
            tenant_id,
        })
    }

    fn purpose_headers(purpose: Option<Purpose>) -> Vec<(&'static str, String)> {
        match purpose {
            Some(purpose) => vec![(PURPOSE_HEADER, purpose.as_str().to_string())],
            None => Vec::new(),
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        purpose: Option<Purpose>,
        body: &RequestBody<'_>,
    ) -> Result<T> {
        let full = format!("{DATA_PREFIX}{path}");
        let headers = Self::purpose_headers(purpose);
        let resp = self
            .api
            .send(method, &full, &headers, body)
            .await
            .map_err(|err| explain_http_error(err, self.tenant_id))?;
        resp.json()
            .await
            .with_context(|| format!("parse response from {full}"))
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        purpose: Option<Purpose>,
    ) -> Result<T> {
        self.request(Method::GET, path, purpose, &RequestBody::None)
            .await
    }

    async fn describe_dataset(
        &self,
        dataset_id: &str,
        version: Option<&str>,
        purpose: Option<Purpose>,
    ) -> Result<DatasetDescription> {
        let path = match version {
            Some(version) => format!(
                "/datasets/{}?version={}",
                urlencoding::encode(dataset_id),
                urlencoding::encode(version)
            ),
            None => format!("/datasets/{}", urlencoding::encode(dataset_id)),
        };
        self.get(&path, purpose).await
    }

    /// Turn `--bind` arguments into the `alias -> version ID` map the API
    /// takes, describing each dataset so the version that will be read is
    /// known before the job is submitted.
    async fn resolve_bindings(
        &self,
        specs: &[BindSpec],
        purpose: Option<Purpose>,
    ) -> Result<BTreeMap<String, String>> {
        let mut bindings = BTreeMap::new();
        for spec in specs {
            let version = match &spec.target {
                BindTarget::Version(version) => version.clone(),
                BindTarget::Dataset {
                    dataset_id,
                    version,
                } => {
                    let description = self
                        .describe_dataset(dataset_id, version.as_deref(), purpose)
                        .await
                        .with_context(|| format!("resolve --bind {}={dataset_id}", spec.alias))?;
                    description.version
                }
            };
            bindings.insert(spec.alias.clone(), version);
        }
        Ok(bindings)
    }

    async fn get_job(&self, job_id: &str) -> Result<QueryJobStatus> {
        self.get(
            &format!("/query-jobs/{}", urlencoding::encode(job_id)),
            None,
        )
        .await
    }

    async fn get_result(&self, job_id: &str, max_rows: Option<u32>) -> Result<QueryResultPage> {
        let mut path = format!("/query-jobs/{}/result", urlencoding::encode(job_id));
        if let Some(max_rows) = max_rows {
            path.push_str(&format!("?max_rows={max_rows}"));
        }
        self.get(&path, None).await
    }

    /// Poll until the job reaches a terminal state or `timeout` elapses.
    async fn wait_for_job(
        &self,
        job: QueryJobStatus,
        interval: Duration,
        timeout: Duration,
        quiet: bool,
    ) -> Result<QueryJobStatus> {
        let started = Instant::now();
        let mut job = job;
        let mut last_state = String::new();
        loop {
            if !quiet && job.state != last_state {
                println!("Query job {}: {}", job.job_id, job.state);
                last_state = job.state.clone();
            }
            if is_terminal(&job.state) {
                return Ok(job);
            }
            if started.elapsed() >= timeout {
                return Err(anyhow!(
                    "query job {} is still {} after {}s. It keeps running on the backend; \
                     check it again with `tachyon data query get {}`.",
                    job.job_id,
                    job.state,
                    timeout.as_secs(),
                    job.job_id
                ));
            }
            tokio::time::sleep(interval).await;
            job = self.get_job(&job.job_id).await?;
        }
    }
}

// ------------------------------------------------------------------ commands

pub async fn run(args: &DataArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = DataApi::new(config, tenant_id)?;
    match &args.command {
        DataCommand::Dataset(dataset) => match &dataset.command {
            DatasetCommand::List {
                limit,
                offset,
                json,
            } => list_datasets(&api, *limit, *offset, *json).await,
            DatasetCommand::Create {
                name,
                file,
                description,
                purpose,
                json,
            } => create_dataset(&api, name, file, description.as_deref(), *purpose, *json).await,
            DatasetCommand::Get {
                dataset_id,
                version,
                purpose,
                json,
            } => get_dataset(&api, dataset_id, version.as_deref(), *purpose, *json).await,
        },
        DataCommand::Query(query) => match &query.command {
            QueryCommand::Run {
                sql,
                binds,
                purpose,
                wait,
                interval_secs,
                timeout_secs,
                max_rows,
                json,
            } => {
                run_query(
                    &api,
                    tenant_id,
                    sql,
                    binds,
                    *purpose,
                    *wait,
                    *interval_secs,
                    *timeout_secs,
                    *max_rows,
                    *json,
                )
                .await
            }
            QueryCommand::Get {
                job_id,
                wait,
                interval_secs,
                timeout_secs,
                max_rows,
                json,
            } => {
                get_query(
                    &api,
                    tenant_id,
                    job_id,
                    *wait,
                    *interval_secs,
                    *timeout_secs,
                    *max_rows,
                    *json,
                )
                .await
            }
        },
    }
}

async fn list_datasets(
    api: &DataApi<'_>,
    limit: Option<u32>,
    offset: Option<u32>,
    json: bool,
) -> Result<()> {
    let mut query = Vec::new();
    if let Some(limit) = limit {
        query.push(format!("limit={limit}"));
    }
    if let Some(offset) = offset {
        query.push(format!("offset={offset}"));
    }
    let path = if query.is_empty() {
        "/datasets".to_string()
    } else {
        format!("/datasets?{}", query.join("&"))
    };

    let page: DatasetPage = api.get(&path, None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&page)?);
        return Ok(());
    }

    if page.items.is_empty() {
        println!("No datasets.");
        return Ok(());
    }

    let headers = vec![
        "DATASET ID".to_string(),
        "NAME".to_string(),
        "LATEST VERSION".to_string(),
        "UPDATED".to_string(),
        "DESCRIPTION".to_string(),
    ];
    let rows: Vec<Vec<String>> = page
        .items
        .iter()
        .map(|item| {
            vec![
                item.id.clone(),
                item.name.clone(),
                item.latest_version
                    .clone()
                    .unwrap_or_else(|| "(no ready version)".to_string()),
                item.updated_at.format("%Y-%m-%d %H:%M:%SZ").to_string(),
                item.description.clone().unwrap_or_default(),
            ]
        })
        .collect();
    print_table(&headers, &rows);
    println!();
    println!("{} shown, {} total", page.items.len(), page.total_count);
    Ok(())
}

async fn create_dataset(
    api: &DataApi<'_>,
    name: &str,
    file: &Path,
    description: Option<&str>,
    purpose: Option<Purpose>,
    json: bool,
) -> Result<()> {
    let metadata = std::fs::metadata(file).with_context(|| format!("read {}", file.display()))?;
    if metadata.len() > MAX_UPLOAD_BYTES {
        return Err(anyhow!(
            "{} is {} bytes; Tachyon Data accepts at most {MAX_UPLOAD_BYTES} bytes per upload",
            file.display(),
            metadata.len()
        ));
    }
    let bytes = std::fs::read(file).with_context(|| format!("read {}", file.display()))?;
    let file_name = file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("upload.csv")
        .to_string();

    let name = name.to_string();
    let description = description.map(str::to_string);
    let build = move || {
        let part = reqwest::multipart::Part::bytes(bytes.clone())
            .file_name(file_name.clone())
            .mime_str("text/csv")
            .expect("text/csv is a valid MIME type");
        let mut form = reqwest::multipart::Form::new()
            .text("name", name.clone())
            .part("file", part);
        if let Some(description) = &description {
            form = form.text("description", description.clone());
        }
        form
    };

    let created: CreatedVersion = api
        .request(
            Method::POST,
            "/datasets",
            purpose,
            &RequestBody::Multipart(&build),
        )
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&created)?);
        return Ok(());
    }

    println!("Dataset ID: {}", created.dataset_id);
    println!("Version ID: {}", created.version_id);
    println!("Rows:       {}", created.row_count);
    println!();
    println!(
        "Query it with: tachyon data query run --sql \"SELECT * FROM t\" --bind t={} --wait",
        created.dataset_id
    );
    Ok(())
}

async fn get_dataset(
    api: &DataApi<'_>,
    dataset_id: &str,
    version: Option<&str>,
    purpose: Option<Purpose>,
    json: bool,
) -> Result<()> {
    let description = api.describe_dataset(dataset_id, version, purpose).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&description)?);
        return Ok(());
    }

    println!("Dataset ID:  {}", description.dataset.id);
    println!("Name:        {}", description.dataset.name);
    if let Some(text) = &description.dataset.description {
        println!("Description: {text}");
    }
    println!("Version:     {}", description.version);
    println!(
        "Updated:     {}",
        description.dataset.updated_at.format("%Y-%m-%d %H:%M:%SZ")
    );
    match &description.sql_dialect {
        Some(dialect) => println!("SQL dialect: {} {}", dialect.name, dialect.version),
        None => println!(
            "SQL dialect: (none — this version is on a backend this deployment cannot query)"
        ),
    }
    if let Some(access) = &description.access {
        println!("Purpose:     {}", access.purpose);
        println!("Row filtered: {}", access.row_filtered);
        if !access.withheld_columns.is_empty() {
            println!(
                "Withheld columns: {} (not listed in the schema below)",
                access.withheld_columns.join(", ")
            );
        }
        println!("Access revision: {}", access.access_revision);
    }

    println!();
    if description.schema.columns.is_empty() {
        println!("No readable columns.");
        return Ok(());
    }
    let headers = vec![
        "COLUMN".to_string(),
        "TYPE".to_string(),
        "NULLABLE".to_string(),
        "DESCRIPTION".to_string(),
    ];
    let rows: Vec<Vec<String>> = description
        .schema
        .columns
        .iter()
        .map(|column| {
            vec![
                column.name.clone(),
                column.r#type.clone(),
                column.nullable.to_string(),
                column.description.clone().unwrap_or_default(),
            ]
        })
        .collect();
    print_table(&headers, &rows);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_query(
    api: &DataApi<'_>,
    tenant_id: &str,
    sql: &str,
    binds: &[String],
    purpose: Option<Purpose>,
    wait: bool,
    interval_secs: u64,
    timeout_secs: u64,
    max_rows: Option<u32>,
    json: bool,
) -> Result<()> {
    let specs = parse_binds(binds)?;
    if specs.is_empty() {
        return Err(anyhow!(
            "a query must bind at least one dataset; pass --bind NAME=DATASET_ID"
        ));
    }

    let bindings = api.resolve_bindings(&specs, purpose).await?;
    let spec = QueryJobSpec {
        sql: sql.to_string(),
        bindings,
        max_rows,
    };
    let body = serde_json::to_value(&spec)?;

    let job: QueryJobStatus = api
        .request(
            Method::POST,
            "/query-jobs",
            purpose,
            &RequestBody::Json(&body),
        )
        .await?;

    finish_job(
        api,
        tenant_id,
        job,
        Some(sql),
        wait,
        interval_secs,
        timeout_secs,
        max_rows,
        json,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn get_query(
    api: &DataApi<'_>,
    tenant_id: &str,
    job_id: &str,
    wait: bool,
    interval_secs: u64,
    timeout_secs: u64,
    max_rows: Option<u32>,
    json: bool,
) -> Result<()> {
    let job = api.get_job(job_id).await?;
    finish_job(
        api,
        tenant_id,
        job,
        None,
        wait,
        interval_secs,
        timeout_secs,
        max_rows,
        json,
    )
    .await
}

/// Report a submitted job, waiting for and printing its result when asked.
///
/// A refused query arrives as a `202` body with `state: "failed"`, so the
/// failure is turned into a non-zero exit here rather than being printed as
/// if the query had been accepted.
#[allow(clippy::too_many_arguments)]
async fn finish_job(
    api: &DataApi<'_>,
    tenant_id: &str,
    job: QueryJobStatus,
    sql: Option<&str>,
    wait: bool,
    interval_secs: u64,
    timeout_secs: u64,
    max_rows: Option<u32>,
    json: bool,
) -> Result<()> {
    let job = if wait && !is_terminal(&job.state) {
        api.wait_for_job(
            job,
            Duration::from_secs(interval_secs.max(1)),
            Duration::from_secs(timeout_secs),
            json,
        )
        .await?
    } else {
        job
    };

    if let Some(failure) = &job.failure {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "job": &job,
                    "provenance": Provenance::build(&job, tenant_id, sql),
                }))?
            );
        }
        return Err(anyhow!(
            "query job {} failed. {}",
            job.job_id,
            explain_failure(failure)
        ));
    }

    if job.state == "cancelled" {
        if json {
            println!("{}", serde_json::to_string_pretty(&json!({ "job": &job }))?);
        }
        return Err(anyhow!("query job {} was cancelled", job.job_id));
    }

    if job.state != "succeeded" {
        // Not waiting, or waiting was not asked for: report where the job is
        // and how to pick it up again.
        if json {
            println!("{}", serde_json::to_string_pretty(&json!({ "job": &job }))?);
            return Ok(());
        }
        println!("Query job: {}", job.job_id);
        println!("State:     {}", job.state);
        println!();
        println!(
            "Not finished yet. Follow it with: tachyon data query get {} --wait",
            job.job_id
        );
        return Ok(());
    }

    let result = api.get_result(&job.job_id, max_rows).await?;
    let provenance = Provenance::build(&job, tenant_id, sql);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "job": &job,
                "provenance": provenance,
                "result": &result,
            }))?
        );
        return Ok(());
    }

    println!();
    if result.schema.columns.is_empty() {
        println!("No columns returned.");
    } else {
        let headers: Vec<String> = result
            .schema
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect();
        let rows: Vec<Vec<String>> = result
            .rows
            .iter()
            .map(|row| row.iter().map(render_cell).collect())
            .collect();
        print_table(&headers, &rows);
    }

    println!();
    println!("{} of {} rows", result.rows.len(), result.total_rows);
    if result.truncated {
        println!("(the backend truncated this result)");
    }
    if result.next_page_token.is_some() {
        println!("(more rows remain; raise --max-rows to read further)");
    }
    println!();
    provenance.print();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    fn http_err(status: u16, body: &str, retry_after: Option<u64>) -> anyhow::Error {
        anyhow::Error::new(HttpError {
            method: "GET".to_string(),
            path: "/data/v1/datasets".to_string(),
            status: StatusCode::from_u16(status).unwrap(),
            body: body.to_string(),
            diagnostics: None,
            retry_after_secs: retry_after,
        })
    }

    #[test]
    fn parses_a_plain_dataset_bind() {
        let spec = parse_bind("sales=ds_123").unwrap();
        assert_eq!(spec.alias, "sales");
        assert_eq!(
            spec.target,
            BindTarget::Dataset {
                dataset_id: "ds_123".to_string(),
                version: None,
            }
        );
    }

    #[test]
    fn parses_a_pinned_version_bind() {
        let spec = parse_bind("sales=ds_123@dsv_456").unwrap();
        assert_eq!(
            spec.target,
            BindTarget::Dataset {
                dataset_id: "ds_123".to_string(),
                version: Some("dsv_456".to_string()),
            }
        );
    }

    #[test]
    fn a_version_id_binds_directly() {
        let spec = parse_bind("sales=dsv_456").unwrap();
        assert_eq!(spec.target, BindTarget::Version("dsv_456".to_string()));
    }

    #[test]
    fn bind_arguments_are_trimmed() {
        let spec = parse_bind(" sales = ds_123 @ dsv_456 ").unwrap();
        assert_eq!(spec.alias, "sales");
        assert_eq!(
            spec.target,
            BindTarget::Dataset {
                dataset_id: "ds_123".to_string(),
                version: Some("dsv_456".to_string()),
            }
        );
    }

    #[test]
    fn rejects_bind_without_equals() {
        let err = parse_bind("sales").unwrap_err().to_string();
        assert!(err.contains("NAME=DATASET_ID"), "{err}");
    }

    #[test]
    fn rejects_empty_alias_dataset_and_version() {
        assert!(parse_bind("=ds_123").is_err());
        assert!(parse_bind("sales=").is_err());
        assert!(parse_bind("sales=@dsv_1").is_err());
        assert!(parse_bind("sales=ds_1@").is_err());
    }

    #[test]
    fn rejects_a_repeated_alias() {
        let args = vec!["sales=ds_1".to_string(), "sales=ds_2".to_string()];
        let err = parse_binds(&args).unwrap_err().to_string();
        assert!(err.contains("more than once"), "{err}");
    }

    #[test]
    fn an_empty_404_reads_as_the_feature_flag_gate() {
        let err = explain_http_error(http_err(404, "", None), "tn_abc");
        let message = err.to_string();
        assert!(
            message.contains("not enabled for tenant tn_abc"),
            "{message}"
        );
        assert!(message.contains("data_platform"), "{message}");
    }

    #[test]
    fn a_404_with_an_envelope_stays_a_not_found() {
        let body = r#"{"code":"NOT_FOUND","message":"dataset ds_x was not found"}"#;
        let err = explain_http_error(http_err(404, body, None), "tn_abc");
        let message = err.to_string();
        assert!(message.contains("dataset ds_x was not found"), "{message}");
        assert!(!message.contains("data_platform"), "{message}");
    }

    #[test]
    fn a_404_with_a_non_json_body_stays_a_not_found() {
        let err = explain_http_error(http_err(404, "upstream route not found", None), "tn_abc");
        let message = err.to_string();
        assert!(message.contains("upstream route not found"), "{message}");
        assert!(!message.contains("data_platform"), "{message}");
    }

    #[test]
    fn a_403_names_the_action_family() {
        let body = r#"{"code":"FORBIDDEN","message":"not permitted"}"#;
        let err = explain_http_error(http_err(403, body, None), "tn_abc");
        assert!(err.to_string().contains("data:*"), "{err}");
    }

    #[test]
    fn a_429_reports_retry_after() {
        let body = r#"{"code":"TOO_MANY_REQUESTS","message":"too many"}"#;
        let err = explain_http_error(http_err(429, body, Some(30)), "tn_abc");
        assert!(err.to_string().contains("Retry after 30s"), "{err}");
    }

    #[test]
    fn a_401_is_left_to_the_shared_auth_diagnostics() {
        let err = explain_http_error(http_err(401, "", None), "tn_abc");
        assert!(err.downcast_ref::<HttpError>().is_some());
    }

    #[test]
    fn query_failure_codes_get_an_explanation() {
        let failure = QueryJobFailure {
            code: "TENANT_QUERY_CONCURRENCY".to_string(),
            message: "too many open jobs".to_string(),
        };
        let explained = explain_failure(&failure);
        assert!(
            explained.contains("TENANT_QUERY_CONCURRENCY"),
            "{explained}"
        );
        assert!(explained.contains("too many open jobs"), "{explained}");
        assert!(explained.contains("about 30s"), "{explained}");

        let unknown = QueryJobFailure {
            code: "SOMETHING_NEW".to_string(),
            message: "unexpected".to_string(),
        };
        assert_eq!(explain_failure(&unknown), "SOMETHING_NEW: unexpected");
    }

    #[test]
    fn terminal_states_are_the_three_that_stop() {
        for state in ["succeeded", "failed", "cancelled"] {
            assert!(is_terminal(state), "{state}");
        }
        for state in ["queued", "running", "reconciling", "cancel_requested"] {
            assert!(!is_terminal(state), "{state}");
        }
    }

    #[test]
    fn the_query_job_body_is_flat() {
        let spec = QueryJobSpec {
            sql: "select 1 from t".to_string(),
            bindings: BTreeMap::from([("t".to_string(), "dsv_1".to_string())]),
            max_rows: None,
        };
        assert_eq!(
            serde_json::to_string(&spec).unwrap(),
            r#"{"sql":"select 1 from t","bindings":{"t":"dsv_1"}}"#
        );
    }

    #[test]
    fn purpose_values_match_the_header_contract() {
        assert_eq!(Purpose::TenantAnalytics.as_str(), "tenant_analytics");
        assert_eq!(Purpose::AgentAssist.as_str(), "agent_assist");
        assert_eq!(Purpose::VendorOperations.as_str(), "vendor_operations");
    }

    #[test]
    fn sql_is_hashed_for_provenance() {
        // The published SHA-256 of "abc", so a change in the hashing shows up
        // here rather than silently altering what a result is cited against.
        assert_eq!(
            sql_sha256("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(sql_sha256("SELECT 1").len(), 64);
        assert_ne!(sql_sha256("SELECT 1"), sql_sha256("SELECT 2"));
    }

    #[test]
    fn cells_render_strings_bare_and_nulls_empty() {
        assert_eq!(render_cell(&Value::String("east".into())), "east");
        assert_eq!(render_cell(&Value::Null), "");
        assert_eq!(render_cell(&json!(150)), "150");
        assert_eq!(render_cell(&json!(true)), "true");
    }
}
