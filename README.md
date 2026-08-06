# Tachyon SDK

Auto-generated multi-language API clients for the [Tachyon Platform](https://github.com/quantum-box/tachyon-apps) REST API, plus a standalone CLI binary.

## CLI

Latest release: **v0.6.30**

### Install

```sh
npm install -g @tachyon-sdk/cli
```

This installs the `tachyon` command and the `tc` alias. The npm package
downloads the matching native binary from the GitHub Release for your
OS/architecture during `postinstall`.

Standalone installer:

```sh
curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | sh
```

Installs `tachyon` to `/usr/local/bin` (or `~/.local/bin` if you lack write permission).

Local development install (build from source):

```sh
sh scripts/dev-install.sh
```

Builds the CLI from the current source and installs a **real-file copy** to
`~/.local/bin/tachyon` (or use `cargo install --path cli --force` for `~/.cargo/bin`).
**Do not** symlink `~/.local/bin/tachyon` into `cli/target/release/` — a symlink into
`target/` breaks when the directory is cleaned (disk cleanup / `cargo clean`) and silently
takes down the CLI, including the CEO-escalation path and `tachyon-browser` (PLT-2636,
2026-07-18 incident).

Supported platforms:

| OS | Architecture | Artifact |
|----|--------------|----------|
| Linux | x86_64 | `tachyon-linux-x86_64.tar.gz` |
| Linux | arm64 | `tachyon-linux-arm64.tar.gz` |
| macOS | arm64 (Apple Silicon) | `tachyon-darwin-arm64.tar.gz` |
| macOS | x86_64 (Intel) | `tachyon-darwin-x86_64.tar.gz` |

If you see `Failed to fetch latest release tag` (GitHub API 403 / rate limit), pass a token:

```sh
curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | GITHUB_TOKEN=<your-token> sh
```

A classic PAT with `public_repo` scope (or any valid GitHub token) is sufficient.

### Authentication profiles (multi-account)

The CLI supports multiple named auth profiles (similar to `aws --profile` /
`gcloud config configurations`). Each profile stores its own access token,
refresh token, and default tenant in `~/.config/tachyon/profiles/<name>.json`.
The active profile is recorded in `~/.config/tachyon/active_profile`.

```sh
# Log in to two separate accounts
tachyon auth login --profile work
tachyon auth login --profile personal

# Inspect registered profiles (active marked with *)
tachyon auth list

# Switch the active profile persistently
tachyon auth use personal

# Override the active profile for one command
tachyon --profile work compute apps list
TACHYON_PROFILE=work tachyon compute apps list

# Log out of a single profile
tachyon auth logout --profile personal
```

Resolution order when picking a profile for a command:

1. `--profile <name>` global flag
2. `TACHYON_PROFILE` env var
3. `~/.config/tachyon/active_profile` file (set by `auth login` / `auth use`)
4. `default`

Existing single-account installs are auto-migrated: an older
`~/.config/tachyon/credentials.json` is copied to `profiles/default.json`
on first use, with the legacy file kept for downgrade safety.

> Phase 2 (macOS Keychain / Linux secret-service / encryption-at-rest) is
> tracked separately. Profiles are currently plaintext JSON with `0o600` perms.

### Profile-specific PM defaults

Project-management defaults can be stored separately for each auth profile.
They are written to `~/.config/tachyon/settings.json`, not to the profile's
credentials JSON, so logging in or refreshing a token does not overwrite them.

```sh
# Human-operated profile: leave Linear issues unassigned by default
tachyon config set pm.no_delegate true --profile admin

# Agent profile: keep automatic Linear delegation and use a default team
tachyon config set pm.no_delegate false --profile agent_app
tachyon config set pm.default_team "Platform Team" --profile agent_app

# Read all settings or one setting for a profile
tachyon config get --profile admin
tachyon config get pm.no_delegate --profile admin

# Remove a profile default
tachyon config unset pm.default_team --profile agent_app
```

PM defaults are resolved in this order:

1. Explicit CLI flags such as `--no-delegate`, `--team`, `--team-id`, or
   `--delegate-id`
2. `TACHYON_PM_NO_DELEGATE` and `TACHYON_PM_DEFAULT_TEAM`
3. The selected profile's settings
4. Existing behavior, including the server-side tenant IaC default team

`TACHYON_PM_NO_DELEGATE` accepts `true` or `false`. An explicit
`--delegate-id` always requests that delegate even when `no_delegate` is true
for the environment or profile. The same defaults apply to `tachyon pm issue`,
`tachyon issue`, and `tachyon linear issue`.

### IaC change-control approval and concurrent-update protection

`tachyon iac apply`, `tachyon iac import-seed`, and `tachyon iac rollback`
accept a change-control approval token through
`TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN` or `--change-control-token`. Prefer the
environment variable so the token does not appear in shell history or process
arguments. For example, in Bash:

```bash
read -rsp 'Change-control approval token: ' TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN
printf '\n'
export TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN

tachyon iac apply --file tachyon.yml
tachyon iac import-seed --file 003-iac-manifests.yaml
tachyon iac rollback --kind CloudApp --name example --revision 3

unset TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN
```

The token is validated locally when supplied, then sent only in the
`x-tachyon-change-control-token` request header. It is not placed in GraphQL
variables, local IaC state, or command output. `import-seed --dry-run` sends no
mutation and does not require a token.

Before the first write, each command reads the latest saved revision for every
target manifest and sends it as `expectedRevision`. If another writer changes a
manifest after that preflight, the server returns a CAS conflict instead of
overwriting the concurrent update; inspect the latest state and rerun the
command to re-plan. Multi-manifest commands can still complete earlier writes
before a later manifest conflicts.

The token option remains optional at the CLI compatibility layer in this
rollout stage. Commands without a token continue to use the existing
server-side tenant and change-control policy behavior.

### Usage

```sh
# Set credentials
export TACHYON_API_URL=https://api.n1.tachy.one
export TACHYON_TENANT_ID=tn_xxxx
export TACHYON_API_KEY=your-api-key

# Show recent build status for an app
tachyon compute status <app-id>
tachyon compute status <app-id> --limit 5

# Fetch build logs (latest build)
tachyon compute logs <app-id>

# Fetch logs for a specific build
tachyon compute logs <app-id> --build-id <build-id>

# Stream logs until the build completes
tachyon compute logs <app-id> --follow

# Watch logs and final status until completion
tachyon compute builds watch <app-id>
tachyon compute builds watch --build-id <build-id>

# Manually create a txcloud preview environment for a branch
tachyon compute preview <app-id> --branch feature/my-change

# Re-fire a pull-request preview build
tachyon compute preview <app-id> --pr 123
tachyon compute builds trigger <app-id> --pr 123

# Preview a compute app GitHub connection update
tachyon compute apps update <app-id> --connection-id <connection-id>

# Apply after verifying the displayed app name and current connection
tachyon compute apps update <app-id> --connection-id <connection-id> --yes

# Clear the connection (empty string semantics are preserved)
tachyon compute apps update <app-id> --connection-id "" --yes

# Create a one-off Linear issue without automatic delegation
tachyon issue create --provider linear --team PLT \
  --title "Unassigned issue" --no-delegate

# Reconcile only one app's production release-check contract.
# Production change-control approval is still required.
tachyon compute apps apply \
  --file tachyon.yml \
  --app <app-name> \
  --environment production \
  --release-checks-only

# Compact JSON Lines for coding agents
tachyon compute builds watch --build-id <build-id> --agent
tachyon compute logs --build-id <build-id> --follow --agent

# Generate a Cloud App feedback report
tachyon compute apps feedback <app-id> \
  --kind bug \
  --severity high \
  --url https://example.txcloud.app \
  "Production page returns 500."

# Reproduce a cloud build locally in Docker (Phase 1: mock fixture)
# See cli/tests/fixtures/mock-build-config.yaml for the expected shape.
tachyon compute builds reproduce <build-id> --mock <path/to/build-config.yaml> --dry-run
tachyon compute builds reproduce <build-id> --mock <path> --source-dir .
```

`compute builds watch` exits with code 0 only when the build succeeds. Failed,
cancelled, and timed-out builds return non-zero so automation can stop early.
`--agent` emits compact JSON Lines and only repeats status when it changes.

> `compute builds reproduce` (PLT-914) fetches the buildspec + environment for
> a cloud build and replays it locally in a CodeBuild-compatible Docker
> container. Phase 1 requires `--mock <path>`; the live build-config endpoint
> (PLT-913) lands in Phase 2.

### Slack notifications

Use `--mention` more than once to mix Slack users, Teams (User Groups),
broadcasts, and ProjectConfig aliases. `ops slack` is an alias for
`ops notify`.

```sh
# Mention a Team by display name or @handle.
tachyon ops notify send --text "Deploy complete" --mention "Platform Team"
tachyon ops slack send --text "Incident detected" --mention @platform-team

# User IDs, Team IDs, broadcasts, and configured aliases can be mixed.
tachyon ops notify send --text "Please investigate" \
  --mention U0123456789 \
  --mention S0123456789 \
  --mention @here \
  --mention on-call

# A raw Slack User Group mention token remains supported.
tachyon ops notify send --text "Release ready" \
  --mention '<!subteam^S0123456789>'
```

Team display names and handles are resolved through the tenant's saved Slack
connection using exact, case-insensitive matching. The CLI never picks the
first or closest candidate: unknown handles/IDs, ambiguous names/handles, and
disabled Teams fail before the Slack notification is sent. Mentions that
resolve to the same Slack target are sent only once.

### Worker runtime

`tachyon worker` replaces the separately distributed `tachyond` binary for
local Coding Job workers.

```sh
# Install or refresh the tachyon CLI first
curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | sh

# Authenticate and select the operator that owns the worker
tachyon auth login --profile work
tachyon auth use work

# Run while this shell is open
tachyon --profile work --tenant-id tn_xxxx worker run

# Install as tachyon-worker.service on a Linux systemd host
sudo tachyon --profile work --tenant-id tn_xxxx worker start --dry-run
sudo tachyon --profile work --tenant-id tn_xxxx worker start

# Operate the local systemd service
sudo tachyon worker status
sudo tachyon worker logs --follow
sudo tachyon worker restart
```

The worker advertises the `containerized_codex` provider by default and uses
Docker to execute claimed Coding Jobs. Runtime knobs are available through CLI
flags or environment variables:

| Variable | Purpose |
| --- | --- |
| `TACHYON_WORKER_ID` | Stable worker identifier. Defaults to `worker-<hostname>`. |
| `TACHYON_WORKER_PROVIDER` | Provider capability. Currently `containerized_codex`. |
| `TACHYON_WORKER_MAX_CONCURRENT_JOBS` | Maximum concurrent jobs advertised to Tachyon Cloud. |
| `TACHYON_WORKER_POLL_INTERVAL_MS` | Poll interval used by `worker run`. |
| `CODEX_CONTAINER_IMAGE` | Docker image used for containerized Codex jobs. |
| `CODEX_CONTAINER_NETWORK` | Docker network used for job containers. |
| `CODEX_CONTAINER_MEMORY` | Docker memory limit, for example `2g`. |

See [docs/worker-runtime.md](docs/worker-runtime.md) for foreground and
systemd operation, installed files, and the E2E checklist.

## Languages

| Language | Directory | Package |
|----------|-----------|---------|
| Rust | `rust/` | `tachyon-sdk` |
| TypeScript | `typescript/` | `@tachyon/sdk` |
| Python | `python/` | `tachyon-sdk` |

## Usage

### Rust

```toml
[dependencies]
tachyon-sdk = { git = "https://github.com/quantum-box/tachyon-sdk", branch = "main" }
```

### TypeScript

```bash
npm install @tachyon/sdk
```

### Python

```bash
pip install tachyon-sdk
```

## SMS Notifications

SDK users can send SMS notifications through Tachyon's notification boundary
without importing AWS SDKs or provider-specific APIs.

```typescript
import {
  Configuration,
  NotificationsApi,
  type SendSmsNotificationRequest,
} from "@tachyon/sdk";

const notifications = new NotificationsApi(
  new Configuration({ basePath: "https://api.n1.tachy.one" }),
);

const request: SendSmsNotificationRequest = {
  phoneNumber: "+15551234567",
  message: "Your verification code is 123456.",
};

await notifications.sendSmsNotification({
  xOperatorId: "tn_xxxx",
  authorization: `Bearer ${process.env.TACHYON_API_KEY}`,
  sendSmsNotificationRequest: request,
});
```

Use E.164 format for `phoneNumber`, for example `+15551234567`. Runtime
delivery can depend on provider account settings such as AWS SNS sandbox mode,
SMS spend limits, origination identity configuration, and destination country
support.

## npm Packages

Additional domain-specific TypeScript SDKs published under the `@tachyon-sdk/*` scope:

| Package | Version | Description |
|---------|---------|-------------|
| [`@tachyon-sdk/cli`](packages/cli) | `0.6.30` | Global npm installer for the Tachyon CLI |
| [`@tachyon-sdk/storekit`](packages/storekit) | `0.3.0` | Commerce SDK: auth, order management (updateStatus/cancel/refund), inventory operations |
| [`@tachyon-sdk/agent`](packages/agent) | — | Agent runtime SDK |
| [`@tachyon-sdk/agent-chat`](packages/agent-chat) | — | Agent chat utilities + bundled skills |
| [`@tachyon-sdk/storage`](packages/storage) | — | Storage SDK |

```bash
npm install @tachyon-sdk/storekit
```

## Agent Skills

Pre-built skill definitions for AI agents are in the `skills/` and
`.agents/skills/` directories.

| Skill | File | Description |
|-------|------|-------------|
| image-gen | [`skills/image-gen.json`](skills/image-gen.json) | Generate AI images via `tachyon image generate` |
| tachyon-cloud | [`.agents/skills/tachyon-cloud/`](.agents/skills/tachyon-cloud/) | Operate Tachyon Cloud Apps with `tachyon compute`, `tachyon.yml`, env vars, build logs, deployments, and user feedback reports |

### Installing Agent Skills

Install agent skills interactively:

```bash
tachyon skills install
```

Install non-interactively into Codex user scope:

```bash
tachyon skills install tachyon-cloud --codex --scope user --non-interactive
```

Install non-interactively into this workspace:

```bash
tachyon skills install tachyon-cloud --codex --scope workspace --non-interactive
```

The Tachyon Cloud skill expects the released Tachyon CLI to be available on `PATH`:

```bash
curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | sh
tachyon login
tachyon compute apps list
```

### Using image-gen with Claude Code

The skill is also available as a Claude Code skill at `.claude/skills/image-gen/`. Install by copying to your `~/.claude/skills/` directory or cloning this repo.

```bash
# Generate an image and save locally
tachyon image generate \
  --prompt "hero banner for a cloud developer platform, dark theme" \
  --model gpt-image-1.5 \
  --quality high \
  --output hero.png

# Generate and upload to Tachyon Storage
tachyon image generate \
  --prompt "minimalist product logo, blue gradient" \
  --model gpt-image-1.5 \
  --size 1024x1024 \
  --storage

# Multiple images
tachyon image generate \
  --prompt "mobile app mockup screenshots" \
  --n 4 \
  --output mockup.png
# → mockup_1.png, mockup_2.png, mockup_3.png, mockup_4.png
```

### Using image-gen as an API tool (Claude API)

```typescript
import { Anthropic } from "@anthropic-ai/sdk";
import imageGenSkill from "@tachyon-sdk/agent-chat/skills/image-gen.json";

const client = new Anthropic();
const response = await client.messages.create({
  model: "claude-opus-4-7",
  max_tokens: 1024,
  tools: [imageGenSkill],
  messages: [{ role: "user", content: "Create a hero image for our SaaS product" }],
});
```

## Regenerating SDKs

When the OpenAPI spec (`openapi.json`) is updated:

```bash
./scripts/generate.sh
```

This requires [openapi-generator-cli](https://openapi-generator.tech/docs/installation/) to be installed.

## Architecture

```
tachyon-apps (private monorepo)
  └── REST endpoints + utoipa annotations
         │
         ▼ export
  openapi.json (this repo)
         │
         ▼ openapi-generator
  ├── rust/        Rust client
  ├── typescript/  TypeScript client
  └── python/      Python client
```

The OpenAPI spec is the single source of truth. SDKs are auto-generated from it.

## StoreKit SDK

StoreKit uses `apps/bakuure-api/bakuure.openapi.yaml` as its OpenAPI source.
Generate a TypeScript StoreKit client with:

```bash
./scripts/generate-storekit.sh
```

By default this writes to `typescript-storekit/`. Override the spec or output path with:

```bash
STOREKIT_OPENAPI_SPEC=/path/to/bakuure.openapi.yaml \
STOREKIT_TYPESCRIPT_OUT=typescript-storekit \
./scripts/generate-storekit.sh
```

## License

MIT © 2026 Quantum Box株式会社
