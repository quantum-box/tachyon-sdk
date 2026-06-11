---
name: tachyon-cloud
description: "Use when working on Tachyon Cloud: creating or updating `tachyon.yml` manifests, setting Cloud App env vars or secrets, running `tachyon compute apps apply`, triggering builds/deployments, checking build status/logs, diagnosing deploy failures, collecting user feedback, or explaining Tachyon CLI usage for Cloud App delivery."
---

# Tachyon Cloud

## Purpose

Help Tachyon Cloud Apps reach a real deployable state. Prefer live CLI/API evidence over code-only assumptions, and keep secrets out of logs, docs, PRs, and chat.

## Start Here

1. Identify the app, tenant/operator, repository, branch, environment, and deployment target.
2. Check the current CLI surface before making assumptions:

```bash
tachyon --version
tachyon compute --help
tachyon compute apps --help
tachyon compute env --help
```

3. If the task is about an existing app, inspect the live registry and recent builds before editing manifests:

```bash
tachyon compute status <app_id> --limit 10
tachyon compute logs <app_id> --follow
```

4. If the task is about setup from a repo checkout, use the repo manifest as the source of truth:

```bash
tachyon init --name <app_name> --framework <framework> --tenant-id <tenant_id> --non-interactive
tachyon compute apps apply -f tachyon.yml --app <app_name> --environment sandbox --dry-run
tachyon compute apps apply -f tachyon.yml --app <app_name> --environment sandbox
```

Use `--api-url`, `--tenant-id`, `--user-id`, or `TACHYON_AUTH_TOKEN` only when the current profile/config does not already point at the intended Tachyon API and operator.

## `tachyon.yml` Workflow

Treat `tachyon.yml` as the desired Cloud App registry/config source for a repository. Before editing, check whether the repo already has `tachyon.yml`, `tachyon.yaml`, `tachyon.preview.yml`, or `tachyon.production.yml`.

Core rules:

- Prefer `kind: CloudApps` with `spec.apps[]` for repo-level manifests, even if the repo currently has one app.
- Include `repository`, `rootDirectory`, `framework`, `deploymentTarget`, and `build` fields explicitly.
- Preserve `rootDirectory` as the app root and `dockerContext` as the Docker build context; do not collapse them into one field.
- Use `--app <name>` when a manifest contains multiple apps.
- Keep app names stable. Changing `spec.apps[].name` can create or target a different Cloud App.
- Put public config in `envVars[].value`; put secret references in `envVars[].valueFrom.secret`.
- Do not put raw secret values, OAuth client secrets, API keys, provider tokens, or workload identity issuer tokens in `tachyon.yml`.
- For environment-specific config, use preview/production overlays or split files (`tachyon.preview.yml`, `tachyon.production.yml`) according to the repository pattern.

When creating or changing `tachyon.yml`, read [references/cloud-app-operations.md](references/cloud-app-operations.md) for the field checklist, concrete YAML examples, supported env var forms, environment overlay behavior, and deployment target notes.

## Env Vars And Secrets

Use manifest `envVars` for desired configuration, and the CLI env API for real values:

```bash
tachyon compute env set <app_id> KEY=value --target production --secret KEY
tachyon compute env set <app_id> PUBLIC_KEY=value --target production
tachyon compute env list <app_id>
```

Rules:

- Treat `KEY=value` command inputs as secrets when the key is passed to `--secret`; otherwise they are registered as plain provider variables.
- Never repeat secret values in the final answer. Report only key names and whether the operation succeeded.
- In `tachyon.yml`, reference secrets with `valueFrom.secret: <vault>/<key>` or the shape already used by the repo.
- Do not add a tenant prefix inside `valueFrom.secret`; the tenant comes from the operator context.
- If apply reports a missing secret ref, register the value through the env API or the platform secret owner path, then re-run dry-run/apply.

## Deploy And Verify

For pull-style Cloud Apps, a successful manifest apply is not the same thing as a deployed build. Verify the build/deployment path:

```bash
tachyon compute status <app_id> --limit 10
tachyon compute logs <app_id> --follow
```

For pull-style apps, trigger and watch a build with `tachyon compute builds trigger`
and `tachyon compute builds watch` when the app is ready to deploy from source.

After deploy, verify the live URL or platform UI when the user asks for behavior confirmation. API-only checks are insufficient for "動作確認" in this repo.

## User Feedback

When a Cloud App user reports a problem, request, or question, use the CLI to generate a consistent feedback report:

```bash
tachyon compute apps feedback <app_id> \
  --kind bug \
  --severity high \
  --url https://example.txcloud.app/path \
  --build-id <build_id> \
  --deployment-id <deployment_id> \
  --message "What the user observed"
```

Use `--json` when the report will be consumed by automation. Use `--metadata KEY=VALUE` for non-secret context such as browser, role, account, or plan. Do not put tokens, passwords, API keys, cookies, or private URLs in feedback metadata.

## Routing Safety

For `txcloud.app` public apps, do not create Cloudflare routes, Pages custom domains, DNS records, or ROUTES KV entries manually as a one-off. Public routing must converge through txcloud-proxy desired state (`apps/txcloud-proxy/routes.json`, Terraform/bootstrap state, and reconcile flow). If the requested feature cannot be expressed there yet, implement the proxy-side capability first.

## Failure Triage

- `401` / `UNAUTHORIZED`: verify login/profile/token. Do not print tokens.
- `403 PermissionDenied`: verify `--tenant-id`, operator access, and saved CLI tenant context.
- Missing secret: register the secret value through Tachyon env/secrets flow, then apply again.
- Build failed: inspect `tachyon compute logs <app_id> --follow`; quote the failing command/error, not secret-bearing env.
- Manifest drift: compare the live Cloud App registry against `tachyon.yml` before hand-editing when the live record is authoritative.
- Production route mismatch: check txcloud-proxy desired state before touching Cloudflare directly.

## Reporting

Summarize the actual command outcome, selected app id/name, target operator/environment, changed fields, deployment/build id if available, and any remaining blocker. Keep secret values redacted.
