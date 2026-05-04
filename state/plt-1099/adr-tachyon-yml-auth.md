# ADR: tachyon.yml auth schema and credential issuance

Status: Draft for review
Date: 2026-05-04
Scope: Design only. No implementation, release, or merge decision is implied by this draft.

## 1. tachyon.yml auth schema proposal

Add a top-level `auth:` block to `tachyon.yml`. The block declares which credentials the app needs; it never stores credential values.

Proposed shape:

```yaml
auth:
  version: 1
  default_provider: tachyon
  providers:
    - name: tachyon
      type: oauth2_client_credentials
      audience: https://api.n1.tachy.one
      expiry_days: 90
      scopes:
        - agent:execute
      storage:
        backend: aws_secrets_manager
        ref: tn_<env>/auth/tachyon
```

Provider entry fields:

| Field | Required | Type | Notes |
| --- | --- | --- | --- |
| `name` | yes | string | Stable provider identifier. Unique within `auth.providers[]`. CLI commands address providers by this name. |
| `type` | yes | enum | Initial enum: `oauth2_client_credentials`, `api_key`, `service_account`. Future values can be added without changing the top-level schema. |
| `audience` | no | string | Token issuance scope or resource target. Required by providers that mint scoped access tokens. |
| `expiry_days` | no | int | Credential lifetime. Default `90`. Local bootstrap may use shorter values. |
| `scopes` | no | string array | App-level permission hints passed to the issuance API when supported. |
| `environment` | no | enum/string | Optional environment selector such as `dev`, `staging`, `prod`; omitted means the active CLI environment/profile decides. |
| `storage.backend` | yes | enum | Initial enum: `aws_secrets_manager`, `cloudflare_secrets_store`, `tachyon_credential_service`, `local_file`. |
| `storage.ref` | yes | string | Pointer to storage location. This is a reference, not a secret value. |
| `metadata` | no | map | Non-sensitive labels such as owner team slug, purpose, or migration tag. |

Single-provider example:

```yaml
apiVersion: tachyon/v1
kind: CloudApp
metadata:
  name: orders-console
  tenant_id: tn_<env>
spec:
  framework: vite
auth:
  version: 1
  providers:
    - name: tachyon
      type: oauth2_client_credentials
      audience: https://api.n1.tachy.one
      expiry_days: 90
      scopes: [agent:execute, compute:deploy]
      storage:
        backend: aws_secrets_manager
        ref: tn_<env>/auth/tachyon
```

Multi-provider example:

```yaml
auth:
  version: 1
  default_provider: tachyon
  providers:
    - name: tachyon
      type: oauth2_client_credentials
      audience: https://api.n1.tachy.one
      expiry_days: 90
      scopes: [agent:execute]
      storage:
        backend: aws_secrets_manager
        ref: tn_<env>/auth/tachyon
    - name: cloudflare_pages
      type: api_key
      expiry_days: 30
      storage:
        backend: aws_secrets_manager
        ref: tn_<env>/providers/cloudflare_pages
```

Minimal local bootstrap example:

```yaml
auth:
  providers:
    - name: tachyon
      type: oauth2_client_credentials
      storage:
        backend: local_file
        ref: .tachyon/credentials.json
```

Existing code fit:

- `cli/src/config/loader.rs` currently parses `tachyon.yml` into `ProjectConfig` with `metadata` only, so adding `auth` is a backward-compatible optional struct expansion.
- `cli/src/commands/init.rs` already refuses to overwrite `tachyon.yml` unless `--force` is set; `tachyon auth init` should reuse that collision posture for auth block mutations.
- `cli/src/auth.rs` already has profile-aware credential storage under the user's config directory. The proposed app credential storage is separate from human login profiles and must not reuse profile token files.

## 2. Credential destination options

| Option | Security | Latency | Cost | Migration effort | Vendor lock-in | Operational complexity |
| --- | --- | --- | --- | --- | --- | --- |
| (a) AWS Secrets Manager | Strong fit for production. Existing infra already uses tenant/provider paths and Terraform-managed secret versions. Supports IAM, KMS, CloudTrail, rotation lifecycle. | Runtime fetch adds network latency unless cached; deployment-time injection can avoid hot-path fetch. | Per-secret and API-call pricing, plus KMS/rotation Lambda costs when custom rotation is enabled. | Low to medium because current infra already uses ASM-style paths such as `tn_<env>/providers/<name>`. | Medium AWS dependency. | Medium; IAM, version staging, and Terraform ownership must be explicit. |
| (b) Cloudflare Workers Analytics Engine or Cloudflare Secrets Store | Secrets Store is appropriate for Worker-readable secrets; WAE is analytics-oriented and should not be the credential SSoT. | Excellent for Workers when bound inside Cloudflare. Poorer for non-Worker runtimes that need Cloudflare API access. | Potentially low for Worker-local use; exact cost depends on Cloudflare plan and beta/product terms. | Medium because existing Tachyon provider secrets are currently mirrored through ASM. | High for Worker-only path. | Medium; deployment/version semantics and product maturity need validation. |
| (c) Tachyon credential service | Best long-term SSoT if it enforces tenant, provider, audit, expiry, and grace-period semantics centrally. | One internal API hop; cacheable by CLI/runtime. | New service and storage costs. | High because service, schema, authz, audit, and incident runbooks must be built. | Low cloud lock-in, higher internal platform dependency. | High initially, lower after mature APIs and runbooks exist. |
| (d) Local `.tachyon/credentials.json` with chmod 600 | Acceptable only for developer bootstrap. Plain local file still risks endpoint compromise and backup leakage. | Fastest local access. | Near zero. | Low. | None. | Low for dev, unacceptable for shared/prod operations. |

Use-case recommendation:

| Environment | Recommended destination | Rationale |
| --- | --- | --- |
| Dev bootstrap | Local `.tachyon/credentials.json` or ASM dev tenant | Local file unblocks iteration; ASM dev tenant is preferred when testing rotation/collision behavior. |
| Staging | AWS Secrets Manager | Matches current infra pattern and exercises IAM, audit, and versioning before production. |
| Production | AWS Secrets Manager for Phase 1; Tachyon credential service as later SSoT candidate | ASM is already operationally present. A Tachyon-owned service should wait until API, audit, and migration semantics are proven. |
| Cloudflare Worker-only deployments | Cloudflare Secrets Store as optional adapter | Use only when the consuming runtime is Cloudflare-first and the secret is not required by AWS/Lambda or non-Worker consumers. |

Existing infra observations:

- `cluster/n1-aws/cloudflare_provider_secrets.tf` reads provider secrets from AWS Secrets Manager with names shaped like `${root_tenant_id}/providers/cloudflare_pages`.
- `cluster/n1-aws/cloudflare_tokens.tf` writes Terraform-minted Cloudflare provider tokens back into existing AWS Secrets Manager secret versions while preserving JSON shape for consumers.
- `cluster/n1-aws/lambda.tf` passes secret paths such as a Cognito OAuth2 secret path to runtime environment variables rather than embedding secret values.
- `openapi.json` already exposes OAuth2 client create, revoke, and rotate endpoints. Generated responses include plain-text secret fields, so the CLI must persist returned material immediately and must not log it.

## 3. rotate API design

CLI command:

```bash
tachyon auth rotate <provider> [--environment <env>] [--grace-period 10m] [--force]
```

Flow:

1. Load `tachyon.yml`, resolve `auth.providers[]`, and require exactly one provider matching `<provider>`.
2. Resolve environment and storage backend from CLI flags, active profile, and provider config.
3. Create or rotate upstream credential.
4. Store the new credential in the configured destination as a new version.
5. Mark old credential as valid during a grace period.
6. Emit audit event with provider name, tenant/environment, actor identity, upstream credential id, storage backend/ref, timestamps, and result. Do not include credential values.
7. Print only non-sensitive identifiers and next steps.

Grace period:

- Default `10m` for OAuth2 client credentials.
- `--grace-period 0` is allowed only with `--force`.
- During the grace period both old and new credentials are accepted. After the period, the old credential is revoked or disabled.

ASM backend:

- Use AWS SDK to read current metadata, then write a new version with `PutSecretValue` or update managed metadata with `UpdateSecret`.
- Preserve existing JSON shape unless schema migration is explicitly requested.
- Respect AWS guidance that frequent `PutSecretValue`/`UpdateSecret` calls create versions; avoid automated tight loops.
- Use version staging or equivalent metadata to support rollback and grace-period cutover.

Cloudflare backend:

- For per-Worker secrets, call Cloudflare secret APIs or Wrangler-equivalent API behavior to put a secret, creating a new Worker version when required.
- For Cloudflare Secrets Store, use account-level secret APIs/bindings once product maturity and API constraints are confirmed.
- Treat Workers Analytics Engine as telemetry for rotation outcomes, not as credential storage.

Tachyon credential service backend:

- Proposed API:

```http
POST /v1/auth/providers/{provider}/credentials
POST /v1/auth/providers/{provider}/credentials/{credential_id}/rotate
POST /v1/auth/providers/{provider}/credentials/{credential_id}/revoke
GET  /v1/auth/providers/{provider}/credentials/{credential_id}
```

- Responses may include a secret value only once on create/rotate. CLI writes it immediately to the configured storage backend and redacts it from logs.
- The service owns audit log, grace-period state, idempotency keys, and tenant authorization.

Rollback:

- If upstream credential creation succeeds but storage write fails, immediately revoke the new upstream credential when the provider supports revocation.
- If storage write succeeds but activation fails, keep the previous credential active and mark the new version as pending/failed.
- If revoking the old credential fails after the new credential is active, return a warning state and require follow-up `tachyon auth rotate --resume <operation_id>`.
- Each rotate run gets an idempotency key and operation id so retries do not mint multiple live credentials accidentally.

## 4. Existing-resource collision behavior

`tachyon.yml` collisions:

- Default: error if `auth.providers[]` already contains the same `name`.
- `--merge`: update non-sensitive config fields only; never alter storage destination unless explicitly provided.
- `--force`: replace the provider entry after showing a diff of non-sensitive fields.
- Provider names are case-sensitive but should be restricted to ASCII letters, digits, `_`, `-`, and `.` to match existing CLI profile-name safety rules.

Secret destination collisions:

- Default: error if the destination already exists and is not already linked to the same provider metadata.
- `--use-existing`: bind `tachyon.yml` to the existing storage ref without rotating material.
- `--force`: rotate/overwrite by creating a new version, preserving previous version for rollback when backend supports it.
- For ASM, preserve existing secret name and JSON shape unless a migration flag is passed.
- For Cloudflare Worker secrets, warn that `secret put` style behavior may create a new Worker version/deploy depending on API path.

`tachyon auth init` behavior:

- Creates `auth:` block when missing.
- Adds one provider by default.
- Does not issue credentials unless `--issue` is passed.
- Does not overwrite existing `tachyon.yml` fields without `--merge` or `--force`.
- Prints storage refs, never material.

Multi-environment behavior:

- Same provider `name` can exist across environments, but each environment must resolve to a distinct `storage.ref`.
- Recommended storage ref shape: `tn_<env>/auth/<app>/<provider>` for app credentials and `tn_<env>/providers/<provider>` for shared provider credentials.
- `tachyon auth rotate tachyon --environment staging` rotates only the staging ref.
- Ambiguous environment resolution must fail closed and ask for `--environment`.

## 5. GO/NO-GO recommendation

Recommendation: GO with AWS Secrets Manager as the first shared/staging/production destination, plus local-file bootstrap for dev only.

Rationale:

- Existing Tachyon infra already uses ASM provider secret paths, Terraform-managed secret versions, and runtime secret-path references.
- ASM gives an acceptable security and audit baseline now while avoiding a new credential service on the critical path.
- The schema keeps storage backend abstract, so a Tachyon credential service can become the SSoT later without changing the top-level `auth.providers[]` contract.

Phased delivery:

| Phase | Deliverables | Completion indicators |
| --- | --- | --- |
| Phase 0: local schema/bootstrap | `auth:` schema parser, `tachyon auth init`, local `.tachyon/credentials.json` backend for dev, redaction tests. | Existing `tachyon.yml` remains compatible; generated config contains only refs; local credential file is gitignored and chmod 600. |
| Phase 1: ASM integration | `tachyon auth issue <provider>` and `tachyon auth rotate <provider>` for ASM, storage ref naming convention, CloudTrail/audit event mapping, collision flags. | Staging provider can be issued, rotated with grace period, and rolled back without printing or committing material. |
| Phase 2: full rotate API and backend adapters | Tachyon credential service API or facade, Cloudflare Secrets Store adapter, idempotent operation ids, centralized audit, migration tooling from existing ASM refs. | Production credential lifecycle is operated through one API surface; old credentials are revoked after grace period; audit records link actor/provider/storage ref without secret values. |

NO-GO conditions before implementation:

- No agreed storage ref naming convention for app credentials.
- No decision on whether OAuth2 client create/rotate endpoints are the immediate issuance API or only an upstream primitive.
- No redaction policy and test coverage for CLI stdout/stderr, PR/taskdocs, and structured logs.
- No explicit HOLD release policy for generated credentials until review confirms that no material is committed or logged.

## Sources

- AWS Secrets Manager User Guide, "What is AWS Secrets Manager?", https://docs.aws.amazon.com/secretsmanager/latest/userguide/intro.html, retrieved 2026-05-04.
- AWS Secrets Manager User Guide, "Modify an AWS Secrets Manager secret", https://docs.aws.amazon.com/secretsmanager/latest/userguide/manage_update-secret.html, retrieved 2026-05-04.
- Cloudflare Workers docs, "Secrets", https://developers.cloudflare.com/workers/configuration/secrets/, retrieved 2026-05-04.
- Cloudflare Secrets Store docs, "Cloudflare Secrets Store", https://developers.cloudflare.com/secrets-store/, retrieved 2026-05-04.
- Local primary-source refs: `cli/src/auth.rs`, `cli/src/config/loader.rs`, `cli/src/commands/init.rs`, `openapi.json`, and read-only infra refs under `tachyon-apps/cluster/n1-aws/`.
