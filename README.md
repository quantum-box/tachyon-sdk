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
export TACHYON_API_URL=https://api.tachyon.run
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

MIT
