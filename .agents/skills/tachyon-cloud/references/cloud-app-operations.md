# Cloud App Operations Reference

## `tachyon.yml` Field Checklist

Use this checklist before writing or reviewing a Cloud App manifest:

```yaml
apiVersion: apps.tachy.one/v1alpha  # required
kind: CloudApps                     # repo-level manifest
metadata:
  name: <repo-or-suite-name>         # stable manifest name
  tenantId: tn_...                   # operator/tenant that owns the apps
spec:
  apps:
    - name: <cloud-app-name>         # stable app name; used for apply matching
      repository:
        url: https://github.com/<owner>/<repo>
        owner: <owner>
        name: <repo>
        defaultBranch: main
      rootDirectory: <app/root>      # app source root inside checkout
      dockerContext: <context>       # optional; only when Docker context differs
      framework: <framework>
      deploymentTarget: <target>
      build:
        command: <build command>
        installCommand: <install command>
        outputDirectory: <dist dir>
        nodeVersion: "20"
      envVars: []
```

Required fields for most apps:

- `apiVersion`: use `apps.tachy.one/v1alpha`.
- `kind`: prefer `CloudApps` for repositories. Use `CloudApp` only when the repo pattern already does.
- `metadata.name`: manifest-level name, usually repo or suite name.
- `metadata.tenantId`: owning operator/tenant. Do not invent this; use the target operator context.
- `spec.apps[].name`: stable Cloud App name.
- `repository`: GitHub repo metadata. Keep it explicit in distributed manifests so another agent can apply the same app without guessing repository context.
- `rootDirectory`: where the app lives in the checkout. Use `""` only for repo-root apps.
- `framework`: examples include `vite`, `next_js`, `worker`, `cargo_lambda`.
- `deploymentTarget`: examples include `cloudflare_pages`, `cloudflare_workers`, `lambda`, `cloud_run`.
- `build`: target-specific build contract.

Optional but common fields:

- `dockerContext`: Docker build context when different from `rootDirectory`.
- `buildspecStrategy`: for repo-managed buildspecs, for example `repo:.codebuild/app_buildspec.yml`.
- `customDomains`: declaration only; public `txcloud.app` routes still follow txcloud-proxy desired state.
- `d1Databases`, `r2Buckets`, `provisionedDatabase`: managed data resources.
- `speedInsights`, `rum`, `middleware`, `auth`: runtime/platform features.
- `environments`: preview/staging/production overrides.

## Minimal CloudApps Manifest

```yaml
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: example-suite
  tenantId: tn_01hjryxysgey07h5jz5wagqj0m
spec:
  apps:
    - name: example-web
      repository:
        url: https://github.com/quantum-box/example
        owner: quantum-box
        name: example
        defaultBranch: main
      rootDirectory: apps/web
      framework: vite
      deploymentTarget: cloudflare_pages
      build:
        command: yarn build
        outputDirectory: dist
        nodeVersion: "20"
      envVars:
        - name: VITE_API_URL
          value: https://api.example.com
        - name: RESEND_API_KEY
          valueFrom:
            secret: contact/RESEND_API_KEY
```

## Target-Specific Examples

### Cloudflare Pages / Vite

```yaml
framework: vite
deploymentTarget: cloudflare_pages
build:
  command: yarn build
  outputDirectory: dist
  nodeVersion: "20"
```

### Cloudflare Workers / Vinext

```yaml
framework: worker
deploymentTarget: cloudflare_workers
build:
  command: yarn vinext:build:prod
  nodeVersion: "20"
```

### Lambda / cargo-lambda

```yaml
framework: cargo_lambda
deploymentTarget: lambda
buildspecStrategy: repo:.codebuild/bakuure_api_lambda_buildspec.yml
build:
  package: bakuure-api
  binary: lambda-bakuure-api
  release: true
  arch: arm64
```

### Cloud Run / Container App

```yaml
framework: next_js
deploymentTarget: cloud_run
rootDirectory: apps/admin-ui
dockerContext: .
build:
  command: yarn build
  nodeVersion: "20"
```

Keep `rootDirectory` as the app root and `dockerContext` as the build context. This matters for monorepos where Docker needs files outside the app directory.

## Env Vars In `tachyon.yml`

Plain public values:

```yaml
envVars:
  - name: NEXT_PUBLIC_BACKEND_API_URL
    value: https://api.n1.tachy.one
```

Secret references:

```yaml
envVars:
  - name: RESEND_API_KEY
    valueFrom:
      secret: contact/RESEND_API_KEY
```

Guidelines:

- `value` is for non-secret values safe to commit.
- `valueFrom.secret` is a `{vault}/{key}` reference, not a raw value.
- Do not include tenant prefixes such as `tn_xxx/contact/RESEND_API_KEY`.
- Keep env var names stable. Removing a txcloud-managed key from the manifest can remove it from the provider on reconcile.
- Provider-only keys that are not declared in `tachyon.yml` should be treated as unmanaged and preserved by Tachyon.
- For secret values, first register or update the actual value through the Tachyon env/secret flow, then commit only the reference.

Internal service references:

```yaml
envVars:
  - name: TACHYON_API_URL
    valueFrom:
      internalService:
        appName: tachyon-api
        field: url
```

Use `valueFrom.internalService` when one Cloud App needs a server-side URL for
another Cloud App in the same platform environment. This is a platform-standard
feature, not an app-specific override.

Guidelines:

- Requires Tachyon CLI `0.6.13` or newer. Released CLI `0.6.12` lacks
  `valueFrom.internalService` apply/preflight support.
- `appName` is the target Cloud App registry name.
- `field` defaults to `url`; omit it unless a future field is documented.
- The CLI preflights references through `/v1/internal-service-refs/preflight`
  before app mutation. A missing app or active deployment fails before manifest
  apply changes are written.
- The committed manifest stores the reference, not a resolved URL. The
  control plane materializes a server-managed env reference and the compute
  runtime resolves it at build/deploy time from active deployment state.
- Do not use `NEXT_PUBLIC_`, `VITE_`, or other browser-exposed variable names
  for internal service URLs.
- Do not replace this with a public `txcloud.app` URL to bypass preflight. That
  breaks deterministic deploy-time resolution and exposes an internal
  dependency through public routing.

## Environment Overlays

Use one Cloud App record with environment-specific build/env config when preview and production differ.

```yaml
envVars:
  - name: NEXT_PUBLIC_API_URL
    value: https://api.example.com
environments:
  preview:
    build:
      command: yarn build:preview
      outputDirectory: dist-preview
    envVars:
      - name: NEXT_PUBLIC_API_URL
        value: https://preview-api.example.com
  production:
    build:
      command: yarn build:production
      outputDirectory: dist
```

Resolution order is:

1. Explicit CLI file: `tachyon compute apps apply -f <path>`
2. `TACHYON_CONFIG`
3. `tachyon.preview.yml` or `tachyon.production.yml` for selected preview/production builds
4. `tachyon.yml`, then `tachyon.yaml`

Use overlays when preview and production need different build commands, output directories, D1/R2 resources, or env vars while retaining one Cloud App identity. If every environment can share the same settings, keep the manifest flat.

## Apply Checklist

Before apply:

1. Confirm the target operator/tenant and API URL.
2. Verify `tachyon.yml` parses as YAML and contains the intended app name.
3. Confirm secrets are references only and actual values are already registered.
4. If the manifest uses `valueFrom.internalService`, confirm the local CLI is
   `0.6.13` or newer:

```bash
tachyon --version
```

5. Run dry-run:

```bash
tachyon compute apps apply -f tachyon.yml --app <app_name> --environment sandbox --dry-run
```

6. Apply only after checking the changed fields:

```bash
tachyon compute apps apply -f tachyon.yml --app <app_name> --environment sandbox
```

After apply, check build/deploy separately with `tachyon compute status` and `tachyon compute logs`; manifest apply alone does not prove the live app is serving correctly.

## Deployment Target Notes

- `cloudflare_pages`: static/frontend output, usually `build.outputDirectory`.
- `cloudflare_workers`: Worker/Vinext-style target; confirm framework and build command in the app manifest.
- `lambda`: cargo-lambda or server target. Include `build.package`, `build.binary`, `build.release`, and `build.arch` when needed.
- `cloud_run`: container/server target. Keep `rootDirectory` and `dockerContext` distinct when Docker context differs from app root.

## CLI Commands

```bash
# Import live Cloud Apps into repo manifest
tachyon init --name <app_name> --framework <framework> --tenant-id <tenant_id> --non-interactive

# Preview and apply manifest
tachyon compute apps apply -f tachyon.yml --app <app_name> --environment sandbox --dry-run
tachyon compute apps apply -f tachyon.yml --app <app_name> --environment sandbox

# Legacy IaC path for generic manifests
tachyon compute apps apply -f tachyon.yml --app <app_name> --dry-run
tachyon compute apps apply -f tachyon.yml --app <app_name>

# Env var management
tachyon compute env set <app_id> SECRET_KEY=value --target production --secret SECRET_KEY
tachyon compute env set <app_id> PUBLIC_URL=https://example.com --target production
tachyon compute env list <app_id>

# Build/deploy observation
tachyon compute status <app_id> --limit 10
tachyon compute logs <app_id> --follow

# Trigger and watch a pull-style build
tachyon compute builds trigger <app_id> --branch main
tachyon compute builds watch <app_id>

# Generate a user feedback report
tachyon compute apps feedback <app_id> --kind bug --severity high --message "User-visible issue"
tachyon compute apps feedback <app_id> --kind feature --message "Please add export" --json
```

## Secret Handling

- Manifest secret references use `{vault}/{key}`, for example `contact/RESEND_API_KEY`.
- Do not use `tn_xxx/contact/RESEND_API_KEY` in `valueFrom.secret`.
- Do not write OAuth client secrets, API keys, workload identity issuer tokens, or provider credentials to git.
- When a command needs `KEY=value`, execute it without echoing the value and summarize it as `KEY` updated.
- Provider-side secret values may appear redacted or empty on readback; do not treat that as drift by itself.

## Workload Identity

For first-party Cloud Apps calling Tachyon APIs, prefer workload identity over distributing API keys:

- Register one binding per app, operator, and environment.
- The app should receive only short-lived runtime tokens.
- Keep `TACHYON_WORKLOAD_IDENTITY_SIGNING_SECRET` and issuer tokens platform-managed.
- Failure modes to check: disabled workload identity, invalid issuer, missing binding, tenant mismatch, and action not allowed.

## txcloud.app Routing

Public `txcloud.app` routing belongs to txcloud-proxy desired state, not direct Cloudflare changes. Check and update the desired state path before reconciling:

- `apps/txcloud-proxy/routes.json`
- Terraform / bootstrap config under `cluster/`
- txcloud-proxy reconcile or deployment runbooks

If live Cloudflare state was changed manually during emergency recovery, record and backfill the desired state immediately.

## User Feedback Reports

Use `tachyon compute apps feedback` from the distributed Tachyon CLI to standardize feedback from users, QA, or operators.

```bash
tachyon compute apps feedback app_01example \
  --kind bug \
  --severity high \
  --url https://example.txcloud.app/orders \
  --build-id bld_01example \
  --deployment-id dep_01example \
  --contact user@example.com \
  --metadata browser=Chrome \
  --metadata plan=pro \
  --message "Orders page shows a blank screen after filtering by date."
```

Output is Markdown by default. Use JSON for automation:

```bash
tachyon compute apps feedback app_01example \
  --kind question \
  --severity low \
  --message "Can this app support SSO?" \
  --json
```

Metadata rules:

- Use metadata for non-secret context only, such as `browser`, `role`, `plan`, `region`, or `screen`.
- Secret-like metadata keys such as `token`, `password`, `api_key`, `private_key`, or `authorization` are rejected.
- Do not include cookies, bearer tokens, OAuth secrets, provider credentials, or customer private data in the message.
- Include `--url`, `--build-id`, and `--deployment-id` when the feedback relates to a live deployment.
