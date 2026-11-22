# Workload Identity Federation

Workload Identity Federation (WIF) lets a CI job or another workload call the
Tachyon API without a long-lived `TACHYON_API_KEY`. The workload presents an
OIDC token from its own identity provider (for example GitHub Actions), and
Tachyon exchanges it for a short-lived access token (60-900 seconds) that acts
as a service account. What the token may do is decided by that service
account's policies.

```text
GitHub Actions OIDC token ──► POST /v1/auth/workload-identities/token/exchange
                                   (appId, operatorId, environment[, action])
                          ◄── Tachyon access token (Bearer, ≤ 15 min)
```

> **Server version.** Leaving out the action needs tachyon-api v0.92.921 or
> later. Older servers reject an exchange without `action`; on those, pass
> `--action` / `TACHYON_WORKLOAD_IDENTITY_ACTION` (the CLI says so when the
> exchange fails this way).

## 1. Configure the trust (once, as a tenant admin)

A **binding** ties a Cloud App environment to a service account and,
optionally, narrows the actions a workload may request. A **federation provider** says which OIDC tokens may
use the binding. The binding must exist before a provider can be added, and
only the service account's creator or a holder of `auth:UpdateServiceAccount`
may add providers.

```sh
export TACHYON_TENANT_ID=tn_...

# Binding: app + environment -> service account. Without --allowed-action
# the binding allows every action (`*`), so the service account's policies
# decide; pass --allowed-action (repeatable) to narrow it.
tachyon auth workload-identity ensure \
  --app-id app_... --environment production \
  --service-account gha.deploy

# Provider: trust GitHub Actions runs of one repository on main.
# --issuer defaults to https://token.actions.githubusercontent.com and
# --audience to tachyon-api/workload-identity.
tachyon auth workload-identity provider add \
  --app-id app_... --environment production \
  --subject-pattern 'repo:my-org/my-repo:ref:refs/heads/main' \
  --claim repository_owner=my-org

tachyon auth workload-identity list
tachyon auth workload-identity provider list
tachyon auth workload-identity provider delete wif_...
```

`--claim claim=value[,value...]` adds a condition on another token claim and
may be repeated. `--replay-protection allow-missing-jti` accepts tokens
without a `jti` claim (the default, `required`, rejects them). Every command
accepts `--json`. `wif` is an alias for `workload-identity`.

The same settings are available in the platform console's Workload Identity
page, which also has a GitHub Actions preset.

## 2. Use it from GitHub Actions

The job needs `permissions: id-token: write`, so GitHub provides
`ACTIONS_ID_TOKEN_REQUEST_URL` and `ACTIONS_ID_TOKEN_REQUEST_TOKEN`.

### Credential mode (recommended)

Set the binding in environment variables and leave `TACHYON_API_KEY` unset.
Every `tachyon` command then exchanges a token on first use, reuses it until
shortly before it expires, and exchanges a new one after a 401.

```yaml
name: deploy
on:
  push:
    branches: [main]

permissions:
  contents: read
  id-token: write   # required for the GitHub OIDC token

jobs:
  deploy:
    runs-on: ubuntu-latest
    env:
      TACHYON_TENANT_ID: tn_...
      TACHYON_WORKLOAD_IDENTITY_APP_ID: app_...
      TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT: production
      # Optional: TACHYON_WORKLOAD_IDENTITY_ACTION: compute:CreateDeployment
      # (required by tachyon-api before v0.92.921)
    steps:
      - uses: actions/checkout@v4
      - run: npm install -g @tachyon-sdk/cli
      - run: tachyon compute deployments create ...
```

### Print a token for other tools

`tachyon auth workload-identity token` prints the access token to stdout
(never to a terminal). Inside GitHub Actions it also emits `::add-mask::` for
the token, so the runner redacts it if a later step prints it.

```yaml
      - name: Get a Tachyon token
        run: |
          TOKEN=$(tachyon auth workload-identity token \
            --app-id app_... --environment production --ttl 600)
          echo "TACHYON_TOKEN=$TOKEN" >> "$GITHUB_ENV"
      - run: |
          curl -fsS -H "Authorization: Bearer $TACHYON_TOKEN" \
            -H "x-operator-id: $TACHYON_TENANT_ID" \
            https://api.n1.tachy.one/v1/compute/apps
```

`--format json` prints `{"tokenType","accessToken","expiresIn"}` instead of
the bare token.

## Other OIDC issuers

Outside GitHub Actions, point the CLI at the OIDC token yourself, for example
a projected Kubernetes service-account token:

```sh
tachyon auth workload-identity token \
  --app-id app_... --environment production \
  --subject-token-file /var/run/secrets/tokens/tachyon
# or: --subject-token-env MY_OIDC_TOKEN
```

The file or variable is read again on every exchange, so a rotated token is
picked up when the CLI refreshes.

## Reference

| Variable | Flag on `token` | Meaning |
|----------|-----------------|---------|
| `TACHYON_WORKLOAD_IDENTITY_APP_ID` | `--app-id` | Cloud App ID of the binding. Setting it turns on credential mode when `TACHYON_API_KEY` is unset. |
| `TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT` | `--environment` | Binding environment (required). |
| `TACHYON_WORKLOAD_IDENTITY_ACTION` | `--action` | Optional. Action to authorize; when given it must be one of the binding's allowed actions. Omitted from the request when unset; the token carries the binding's allowed actions either way. Required by tachyon-api before v0.92.921. |
| `TACHYON_WORKLOAD_IDENTITY_AUDIENCE` | `--audience` | Audience requested from GitHub Actions (default `tachyon-api/workload-identity`). |
| `TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_FILE` | `--subject-token-file` | Read the OIDC token from a file. |
| `TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV` | `--subject-token-env` | Read the OIDC token from the named variable. |
| `TACHYON_WORKLOAD_IDENTITY_TTL` | `--ttl` | Requested lifetime in seconds; the server clamps it to 60-900. |
| `TACHYON_TENANT_ID` | `--tenant-id` | Tenant **ID** (`tn_...`). Names and aliases cannot be resolved before authentication. |

Precedence: `--api-key` / `TACHYON_API_KEY` > workload identity > stored
login profile. With an API key set, the workload identity variables are
ignored and behavior is unchanged.

The Rust SDK exposes the same exchange as `tachyon_sdk::workload_identity`
(feature `workload-identity`): `exchange_token`,
`fetch_github_actions_id_token`, `SubjectTokenSource`, and the caching
`WorkloadIdentityCredentials`.
