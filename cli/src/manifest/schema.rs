//! `tachyon manifest schema` — print a JSON Schema for supported manifest
//! kinds so agents and editors can read the current spec from the installed
//! binary instead of relying on hand-written docs.
//!
//! The Cloud Apps types below mirror the authoritative server-side spec:
//! - `packages/iac/src/domain/cloud_app_manifest.rs` (quantum-box/tachyon-apps)
//! - `packages/compute/domain/src/cloud_app.rs` (quantum-box/tachyon-apps)
//!
//! Keep this module in sync when the server spec changes. Longer term the
//! schema should be generated in tachyon-apps from those types and served
//! through the API; this module is the interim, CLI-local source.

use anyhow::Result;
use clap::{Args, ValueEnum};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::commands::auth::manifest::AuthManifest;

#[derive(Debug, Clone, Args)]
pub struct SchemaArgs {
    /// Manifest kind to describe
    #[arg(long, value_enum, default_value_t = SchemaKind::CloudApps)]
    pub kind: SchemaKind,
    /// Emit compact JSON instead of pretty-printed output
    #[arg(long)]
    pub compact: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SchemaKind {
    /// Cloud Apps manifest (kind: CloudApps / CloudApp)
    CloudApps,
    /// Auth manifest (custom actions and policies)
    Auth,
}

pub(crate) fn run(args: &SchemaArgs) -> Result<()> {
    let schema = match args.kind {
        SchemaKind::CloudApps => schemars::schema_for!(CloudAppsDocument),
        SchemaKind::Auth => schemars::schema_for!(AuthManifest),
    };
    let output = if args.compact {
        serde_json::to_string(&schema)?
    } else {
        serde_json::to_string_pretty(&schema)?
    };
    println!("{output}");
    Ok(())
}

/// A Cloud Apps manifest document.
///
/// Two document forms are accepted in `tachyon.yml` (multi-document YAML is
/// supported): `kind: CloudApps` declaring several apps under `spec.apps[]`,
/// and `kind: CloudApp` declaring a single app whose `spec` is the app spec
/// itself. `OAuth2Client` documents may accompany them to satisfy
/// `valueFrom.oauth2ClientRef` references.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(title = "CloudAppsManifestDocument")]
pub enum CloudAppsDocument {
    /// Repo-level manifest declaring multiple apps (`kind: CloudApps`).
    CloudApps(Box<CloudAppsManifest>),
    /// Single-app manifest (`kind: CloudApp`).
    CloudApp(Box<CloudAppManifest>),
}

/// Repo-level Cloud Apps manifest (`kind: CloudApps`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CloudAppsManifest {
    /// Manifest API version. Always `apps.tachy.one/v1alpha`.
    pub api_version: CloudAppsApiVersion,
    pub kind: CloudAppsKind,
    pub metadata: CloudAppsMetadata,
    pub spec: CloudAppsSpec,
}

/// Single-app Cloud App manifest (`kind: CloudApp`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CloudAppManifest {
    /// Manifest API version. Always `apps.tachy.one/v1alpha`.
    pub api_version: CloudAppsApiVersion,
    pub kind: CloudAppKind,
    pub metadata: CloudAppsMetadata,
    pub spec: CloudAppSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CloudAppsApiVersion {
    #[serde(rename = "apps.tachy.one/v1alpha")]
    V1Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CloudAppsKind {
    CloudApps,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CloudAppKind {
    CloudApp,
}

/// Manifest metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CloudAppsMetadata {
    /// Manifest name. For `kind: CloudApp` this is also the app name.
    pub name: String,
    /// Tenant that owns the declared apps (e.g. `tn_01hj...`). When omitted
    /// the CLI's active tenant is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// Cross-document ordering hints, e.g. `["OAuth2Client/my-client"]`.
    /// Referenced documents are applied first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CloudAppsSpec {
    /// Declared apps. Each entry is an app name plus a Cloud App spec.
    pub apps: Vec<CloudAppsEntry>,
}

/// Single app entry inside `kind: CloudApps`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CloudAppsEntry {
    /// App name, unique within the tenant.
    pub name: String,
    #[serde(flatten)]
    pub spec: CloudAppSpec,
}

/// Cloud App spec: repository, build, deployment, env vars, and managed
/// resources for one app.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CloudAppSpec {
    /// Source repository. Required for apps built from GitHub.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySpec>,
    /// Directory (relative to repo root) containing the app.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_directory: Option<String>,
    /// Docker build context directory for `framework: docker`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docker_context: Option<String>,
    /// Application framework. Defaults to `next_js`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<Framework>,
    /// Deployment target. Defaults to `cloud_run`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_target: Option<DeploymentTarget>,
    /// Lambda published version retention count. Omitted uses the compute
    /// default; values below the compute minimum are clamped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_retention: Option<u16>,
    /// Service tier. `enterprise` opts Lambda apps into a dedicated network
    /// profile (e.g. TiDB Serverless PrivateLink subnet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<CloudAppTier>,
    /// Named network subnet profile for enterprise-tier apps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,
    /// Build configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildSpec>,
    /// Only trigger auto-builds when changed files match these path globs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watch_paths: Option<Vec<String>>,
    /// Skip auto-builds when all changed files match these path globs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths_ignore: Option<Vec<String>>,
    /// Buildspec source strategy, e.g. `repo:.codebuild/my_buildspec.yml`
    /// to use a buildspec checked into the repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buildspec_strategy: Option<String>,
    /// Environment variables. `type: credential` entries are resolved via
    /// `valueFrom` and stored in Secrets Manager; plain entries are set
    /// directly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_vars: Vec<EnvVarSpec>,
    /// Third-party integrations that inject build/runtime config.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrations: Option<IntegrationsSpec>,
    /// Declarative managed resources (e.g. Sentry projects).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<ResourceSpec>,
    /// Cloudflare D1 database bindings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub d1_databases: Vec<D1DatabaseSpec>,
    /// Managed relational database provisioned at deploy time
    /// (`provider: tidb`) with connection URL secret injection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisioned_database: Option<ProvisionedDatabaseSpec>,
    /// Cloudflare R2 bucket bindings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub r2_buckets: Vec<R2BucketSpec>,
    /// Opt-in Speed Insights (Core Web Vitals collection). Only honored for
    /// static deployment targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed_insights: Option<SpeedInsightsSpec>,
    /// Opt-in Real User Monitoring. Only honored for static deployment
    /// targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rum: Option<RumSpec>,
    /// Declarative edge middleware evaluated by txcloud-proxy before the app
    /// origin receives a request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub middleware: Option<MiddlewareConfig>,
    /// End-user authentication evaluated by txcloud-proxy before the app
    /// origin receives a request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    /// Production liveness probe evaluated by tachyon-reconcile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liveness_proof: Option<ProofConfig>,
    /// Production readiness probe evaluated by tachyon-reconcile before a
    /// deployment is considered complete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readiness_proof: Option<ProofConfig>,
    /// Deploy lifecycle hooks. `preDeploy` hooks run before the provider
    /// deploy; `postDeploy` hooks run after a successful deployment. Any
    /// hook failure fails the deployment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<DeployHooksSpec>,
    /// Custom domains attached to a Cloudflare Pages app.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_domains: Vec<CustomDomainSpec>,
    /// Environment-specific overrides (preview / staging / production).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environments: Option<EnvironmentOverrides>,
}

/// Source repository settings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySpec {
    /// Repository URL, e.g. `https://github.com/quantum-box/tachyon-apps`.
    pub url: String,
    /// Repository owner (GitHub org or user).
    pub owner: String,
    /// Repository name.
    pub name: String,
    /// Branch that production builds track. Defaults to `main`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
}

/// Supported application frameworks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    /// Next.js app.
    NextJs,
    /// Prebuilt static site; `build.outputDirectory` is served as-is.
    Static,
    /// Dockerfile-based app (use with `deploymentTarget: cloud_run`).
    Docker,
    /// Rust HTTP server built with `cargo build --release`. On Lambda it
    /// runs behind the Lambda Web Adapter; Cloudflare Pages is not
    /// supported.
    RustServer,
    /// Native Rust AWS Lambda built with `cargo lambda` (produces a
    /// `bootstrap` binary). Use `build.package` / `build.binary`.
    CargoLambda,
    /// Vite app built to static output.
    Vite,
    Remix,
    Astro,
    CreateReactApp,
    /// Cloudflare Workers script (raw Workers or edge frameworks such as
    /// Hono). Only valid with `deploymentTarget: cloudflare_workers`.
    Worker,
    SvelteKit,
    Nuxt,
    /// Project deployed as serverless functions (Vercel/Netlify-style).
    /// Deployed to Lambda by default.
    ServerlessFunctions,
    /// Cloudflare's Vite-based Next.js reimplementation; smaller bundles
    /// than next-on-pages. Deployed to Cloudflare via wrangler.
    Vinext,
}

/// Where the application is deployed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentTarget {
    CloudRun,
    CloudflarePages,
    Lambda,
    /// Cloudflare Workers (edge functions). Only valid with
    /// `framework: worker`.
    CloudflareWorkers,
    /// AWS ECS. Apps on ECS are deployed externally; the Cloud App registry
    /// only records the target for visibility.
    Ecs,
}

/// Cloud App service tier.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CloudAppTier {
    Standard,
    Enterprise,
}

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildSpec {
    /// Build runner backend. Aliases such as `aws_codebuild`, `k8s-kata`,
    /// and `hetzner-k3s-kata` are normalized by the CLI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_backend: Option<BuildRunnerBackend>,
    /// Build command, e.g. `pnpm run build`. For `framework: cargo_lambda`
    /// it is derived from `package` / `binary` / `release` / `arch` when
    /// omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Dependency install command, e.g.
    /// `pnpm install --filter my-app... --frozen-lockfile`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_command: Option<String>,
    /// Build output directory (relative to repo root) for static-style
    /// deployments, e.g. `apps/my-app/dist`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_directory: Option<String>,
    /// Node.js major version, e.g. `"22"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_version: Option<String>,
    /// Cargo package to build (`framework: cargo_lambda` only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Cargo binary name (`framework: cargo_lambda` only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary: Option<String>,
    /// Build in release mode. Defaults to `true`
    /// (`framework: cargo_lambda` only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<bool>,
    /// Target architecture, `arm64` (default) or `x86_64`
    /// (`framework: cargo_lambda` only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

/// Explicit Cloud App build runner backend.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildRunnerBackend {
    Codebuild,
    KubernetesKata,
}

/// Single environment variable entry.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvVarSpec {
    /// Environment variable name, e.g. `TACHYON_CLIENT_ID`.
    pub name: String,
    /// `credential` resolves the value via `valueFrom` and stores it in
    /// Secrets Manager. Defaults to `plain`.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub env_type: Option<EnvVarType>,
    /// Static plain-text value. Not allowed for `type: credential`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Dynamic value resolved from another IaC resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_from: Option<ValueFromSpec>,
    /// Deployment scope the variable applies to: `production`, `preview`,
    /// or `all` (CLI apply only; defaults to the apply environment).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<EnvVarTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum EnvVarType {
    Credential,
    Plain,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvVarTarget {
    Production,
    Preview,
    All,
}

/// Source for a dynamically resolved env var value. Exactly one reference
/// field should be set.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValueFromSpec {
    /// Reference to an OAuth2Client manifest's credentials.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth2_client_ref: Option<OAuth2ClientRef>,
    /// Reference to a storage provider's configuration field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_ref: Option<StorageRef>,
    /// Reference to a TiDB database credential field stored in Secrets
    /// Manager (manual provisioning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database_ref: Option<DatabaseRef>,
    /// Reference to a tenant-scoped secret vault key registered with
    /// `tachyon compute env set`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<SecretRef>,
    /// Reference to another Cloud App's active deployment URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute_deployment_ref: Option<ComputeDeploymentRef>,
    /// Server-side internal service origin of another Cloud App (never
    /// falls back to public txcloud.app URLs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_service: Option<InternalServiceRef>,
}

/// Reference to a specific field in an OAuth2Client manifest's stored
/// secret.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2ClientRef {
    /// Name of the OAuth2Client manifest (matches its `metadata.name`).
    pub name: String,
    /// Field to extract: `clientId`, `clientSecret`, or `userPoolId`.
    pub field: String,
    /// Redirect URIs used when the reference auto-provisions the client.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redirect_uris: Vec<String>,
    /// Allowed OAuth2 scopes used for auto-provisioning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_scopes: Vec<String>,
    /// Allowed grant types used for auto-provisioning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grant_types: Vec<String>,
    /// Create the client in Tachyon's shared Cognito user pool. Defaults to
    /// `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_tachyon_user_pool: Option<bool>,
}

/// Reference to a storage provider's configuration field.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageRef {
    /// Provider name in ProjectConfig, e.g. `cloudflare_r2` or
    /// `cloudflare_images`.
    pub name: String,
    /// Field to extract: `bucket`, `endpoint`, `access_key_id`,
    /// `secret_access_key`, `account_id`, `api_token`, `delivery_url`,
    /// `provider`, or `region`.
    pub field: String,
}

/// Reference to a TiDB database credential field. The connection secret is
/// expected at `{tenantId}/{name}/tidb` in Secrets Manager.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRef {
    /// Database name used to locate the secret, e.g. `tidb_myapp_prod`.
    pub name: String,
    /// Field to extract from the secret: `url` (connection URL).
    pub field: String,
}

/// Reference to a tenant-scoped secret vault key. The string form
/// (`secret: contact/RESEND_API_KEY`) and the object form
/// (`secret: { path: contact/RESEND_API_KEY }`) are both accepted.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", untagged)]
pub enum SecretRef {
    Path(String),
    Object {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        field: Option<String>,
    },
}

/// Reference to another Cloud App's deployment field, e.g.
/// `{ appName: library-api, field: publicUrl }`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComputeDeploymentRef {
    pub app_name: String,
    pub field: String,
}

/// Reference to a Cloud App internal service origin, e.g.
/// `{ appName: library-api }`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InternalServiceRef {
    pub app_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

/// Third-party integrations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationsSpec {
    /// Sentry project backing this app. Injects `SENTRY_DSN` /
    /// `NEXT_PUBLIC_SENTRY_DSN` from the connected Sentry integration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentry: Option<SentryIntegrationSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SentryIntegrationSpec {
    /// Sentry project name.
    pub project: String,
}

/// Declarative managed resource requested by the app.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ResourceSpec {
    /// Managed Sentry project.
    #[serde(rename = "sentry")]
    Sentry(SentryResourceSpec),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SentryResourceSpec {
    /// Resource name.
    pub name: String,
    pub config: SentryResourceConfigSpec,
    /// Env vars injected from the managed resource.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_inject: Vec<ResourceEnvInjectSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SentryResourceConfigSpec {
    pub organization: String,
    pub team: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_rules: Option<bool>,
    /// DSN kind requested for injection: `public` or `secret`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dsn: Option<SentryDsnKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SentryDsnKind {
    Public,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceEnvInjectSpec {
    /// Env var name to inject.
    pub key: String,
    /// Resource field to inject: `dsn.public` or `dsn.secret`.
    pub value_from: ResourceEnvValueFrom,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ResourceEnvValueFrom {
    #[serde(rename = "dsn.public")]
    DsnPublic,
    #[serde(rename = "dsn.secret")]
    DsnSecret,
}

/// Cloudflare D1 database binding.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct D1DatabaseSpec {
    /// Worker binding name.
    pub binding: String,
    pub database_name: String,
    /// Migrations directory. Defaults to `migrations`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migrations_dir: Option<String>,
}

/// Managed relational database provisioned at deploy time.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionedDatabaseSpec {
    /// Provisioning provider. Currently only `tidb`.
    pub provider: ProvisionedDatabaseProvider,
    /// SQL engine exposed to the app. Currently only `mysql`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<ProvisionedDatabaseEngine>,
    /// Env var name that receives the connection URL. Defaults to
    /// `DATABASE_URL`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env_var: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionedDatabaseProvider {
    Tidb,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionedDatabaseEngine {
    Mysql,
}

/// Cloudflare R2 bucket binding.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct R2BucketSpec {
    /// Worker binding name.
    pub binding: String,
    pub bucket_name: String,
}

/// Speed Insights (Core Web Vitals collection) switch.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpeedInsightsSpec {
    pub enabled: bool,
}

/// Real User Monitoring switch.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RumSpec {
    pub enabled: bool,
}

/// Declarative edge middleware evaluated by txcloud-proxy.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MiddlewareConfig {
    pub enabled: bool,
    /// Only `declarative` is supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<MiddlewareMode>,
    /// Behavior when middleware evaluation fails. Defaults to `closed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_policy: Option<MiddlewareFailurePolicy>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<MiddlewareRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MiddlewareMode {
    Declarative,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MiddlewareFailurePolicy {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MiddlewareRule {
    /// Request path pattern (must start with `/`).
    pub path: String,
    pub action: MiddlewareAction,
    /// Redirect/rewrite destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    /// Redirect status code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub request_headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub response_headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MiddlewareAction {
    Next,
    Redirect,
    Rewrite,
}

/// End-user authentication evaluated by txcloud-proxy. When enabled, an
/// OAuth2 client (`{appName}-web`) and its env vars are auto-provisioned.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub enabled: bool,
    /// Only `tachyon` is supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<AuthProvider>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login_url: Option<String>,
    /// Paths that bypass authentication.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_paths: Vec<String>,
    /// Session cookie names checked by the proxy. Defaults include
    /// next-auth and txcloud session cookies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub session_cookie_names: Vec<String>,
    /// Env var names that receive the auto-provisioned OAuth2 client
    /// credentials.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<AuthEnvConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthProvider {
    Tachyon,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthEnvConfig {
    /// Defaults to `TACHYON_CLIENT_ID`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Defaults to `TACHYON_CLIENT_SECRET`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// Defaults to `TACHYON_USER_POOL_ID`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_pool_id: Option<String>,
}

/// Liveness/readiness probe configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProofConfig {
    /// HTTP path to probe (must start with `/`).
    pub path: String,
    /// Expected HTTP status code. Defaults to `200`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_status: Option<u16>,
    /// Optional exact response body match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_body: Option<String>,
}

/// Deploy lifecycle hooks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeployHooksSpec {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_deploy: Vec<DeployHookSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_deploy: Vec<DeployHookSpec>,
}

/// A single deploy lifecycle hook. Exactly one of `command` or
/// `lambdaInvoke` should be set.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeployHookSpec {
    pub name: String,
    /// Shell command executed by the deploy runner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<DeployCommandHookSpec>,
    /// Lambda invocation hook.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lambda_invoke: Option<DeployLambdaInvokeHookSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_vars: Vec<EnvVarSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeployCommandHookSpec {
    pub run: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeployLambdaInvokeHookSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_name: Option<String>,
    /// Invoke the Lambda backing this Cloud App by name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qualifier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_type: Option<String>,
}

/// Custom domain attached to a Cloudflare Pages app.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomDomainSpec {
    pub hostname: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
}

/// Environment-specific overrides. Each overlay accepts the same fields as
/// the app spec (except `environments`); set fields replace the base value
/// for that runtime environment.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<Box<CloudAppSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staging: Option<Box<CloudAppSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub production: Option<Box<CloudAppSpec>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_apps_schema_lists_all_frameworks_and_targets() {
        let schema = schemars::schema_for!(CloudAppsDocument);
        let rendered = serde_json::to_string(&schema).unwrap();
        for framework in [
            "next_js",
            "static",
            "docker",
            "rust_server",
            "cargo_lambda",
            "vite",
            "remix",
            "astro",
            "create_react_app",
            "worker",
            "svelte_kit",
            "nuxt",
            "serverless_functions",
            "vinext",
        ] {
            assert!(
                rendered.contains(&format!("\"{framework}\"")),
                "schema is missing framework {framework}"
            );
        }
        for target in [
            "cloud_run",
            "cloudflare_pages",
            "lambda",
            "cloudflare_workers",
            "ecs",
        ] {
            assert!(
                rendered.contains(&format!("\"{target}\"")),
                "schema is missing deployment target {target}"
            );
        }
    }

    #[test]
    fn auth_schema_generates() {
        let schema = schemars::schema_for!(AuthManifest);
        let rendered = serde_json::to_string(&schema).unwrap();
        assert!(rendered.contains("actions"));
        assert!(rendered.contains("policies"));
    }

    #[test]
    fn typed_model_accepts_real_world_manifest() {
        // Shape mirrors a real repo-level tachyon.yml: a Workers frontend
        // with credential env vars plus a cargo_lambda API with an
        // environment overlay.
        let yaml = r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: example-apps
  tenantId: tn_01example000000000000000000
spec:
  apps:
  - name: example-console
    repository:
      url: https://github.com/example-org/example-apps
      owner: example-org
      name: example-apps
      defaultBranch: main
    rootDirectory: apps/console
    framework: worker
    deploymentTarget: cloudflare_workers
    build:
      installCommand: pnpm install --filter console... --frozen-lockfile
      command: pnpm run build
      nodeVersion: '22'
    envVars:
    - name: OPEN_NEXT_DEPLOY
      value: 'true'
    - name: DATABASE_URL
      type: credential
      valueFrom:
        databaseRef:
          name: tidb_example_prod
          field: url
    - name: EXAMPLE_API_KEY
      type: credential
      valueFrom:
        secret: example/EXAMPLE_API_KEY
    readinessProof:
      path: /api/health
  - name: example-api
    repository:
      url: https://github.com/example-org/example-apps
      owner: example-org
      name: example-apps
      defaultBranch: main
    rootDirectory: ''
    framework: cargo_lambda
    deploymentTarget: lambda
    buildspecStrategy: repo:.codebuild/example_api_lambda_buildspec.yml
    build:
      runnerBackend: kubernetes_kata
      package: example-api
      binary: lambda-example-api
      release: true
      arch: arm64
    environments:
      preview:
        build:
          runnerBackend: codebuild
"#;
        let document: CloudAppsDocument = serde_yaml::from_str(yaml).unwrap();
        let CloudAppsDocument::CloudApps(manifest) = document else {
            panic!("expected kind: CloudApps document");
        };
        assert_eq!(manifest.spec.apps.len(), 2);
        assert!(matches!(
            manifest.spec.apps[0].spec.framework,
            Some(Framework::Worker)
        ));
        assert!(matches!(
            manifest.spec.apps[1]
                .spec
                .build
                .as_ref()
                .unwrap()
                .runner_backend,
            Some(BuildRunnerBackend::KubernetesKata)
        ));
    }

    #[test]
    fn typed_model_accepts_single_cloud_app_document() {
        let yaml = r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApp
metadata:
  name: example-app
  tenantId: tn_01example000000000000000000
spec:
  envVars:
  - name: TACHYON_CLIENT_ID
    type: credential
    valueFrom:
      oauth2ClientRef:
        name: example-app-admin
        field: clientId
"#;
        let document: CloudAppsDocument = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(document, CloudAppsDocument::CloudApp(_)));
    }
}
