# Tachyon SDK

Auto-generated multi-language API clients for the [Tachyon Platform](https://github.com/quantum-box/tachyon-apps) REST API, plus a standalone CLI binary.

## CLI

Latest release: **v0.2.1**

### Install

```sh
curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | sh
```

Installs `tachyon` to `/usr/local/bin` (or `~/.local/bin` if you lack write permission).

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

### Worker daemon

`tachyon worker` replaces the separately distributed `tachyond` binary for
local Tool Job workers.

```sh
# Install or refresh the tachyon CLI first
curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | sh

# Authenticate and select the operator that owns the worker
tachyon auth login --profile work
tachyon auth use work

# Preview the systemd unit and environment file
sudo tachyon --profile work --tenant-id tn_xxxx worker start --dry-run

# Install and start the worker as tachyon-worker.service
sudo tachyon --profile work --tenant-id tn_xxxx worker start

# Run in the foreground instead of systemd
tachyon --profile work --tenant-id tn_xxxx worker run
```

The worker advertises the `containerized_codex` provider by default and uses
Docker to execute claimed Tool Jobs. Runtime knobs are available through CLI
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
| [`@tachyon-sdk/storekit`](packages/storekit) | `0.3.0` | Commerce SDK: auth, order management (updateStatus/cancel/refund), inventory operations |
| [`@tachyon-sdk/agent`](packages/agent) | — | Agent runtime SDK |
| [`@tachyon-sdk/agent-chat`](packages/agent-chat) | — | Agent chat utilities + bundled skills |
| [`@tachyon-sdk/storage`](packages/storage) | — | Storage SDK |

```bash
npm install @tachyon-sdk/storekit
```

## Agent Skills

Pre-built skill definitions for AI agents are in the `skills/` directory.

| Skill | File | Description |
|-------|------|-------------|
| image-gen | [`skills/image-gen.json`](skills/image-gen.json) | Generate AI images via `tachyon image generate` |

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
