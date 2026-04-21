# Tachyon SDK

Auto-generated multi-language API clients for the [Tachyon Platform](https://github.com/quantum-box/tachyon-apps) REST API, plus a standalone CLI binary.

## CLI

### Install

```sh
curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | sh
```

Installs `tachyon` to `/usr/local/bin` (or `~/.local/bin` if you lack write permission).

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
```

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

## License

MIT © 2026 Quantum Box株式会社
