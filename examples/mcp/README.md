# `tachyon mcp` — sample configurations

`tachyon mcp serve` runs the Tachyon CLI as a [Model Context Protocol]
(https://modelcontextprotocol.io/) server. The first tool exposed is
`generate_image`, which calls OpenAI's GPT Image model directly.

> **Required env**: `OPENAI_API_KEY`. If unset, `generate_image` is **not
> registered** (server still starts, `tools/list` returns `[]`, a warning is
> printed to stderr).

## Transports

| Transport | Use-case | Spec |
|---|---|---|
| `stdio` (default) | Claude Code, Cursor (local install) | MCP 2025-06-18 |
| `http` (Streamable HTTP) | remote / Cursor remote / Cloud deploy | MCP 2025-06-18 |

## Claude Code (stdio)

Register the server (one-time):

```bash
claude mcp add tachyon -s user \
  -e OPENAI_API_KEY=sk-... \
  -- /usr/local/bin/tachyon mcp serve
```

Or in `~/.claude.json` directly:

```jsonc
{
  "mcpServers": {
    "tachyon": {
      "command": "/usr/local/bin/tachyon",
      "args": ["mcp", "serve"],
      "env": { "OPENAI_API_KEY": "sk-..." }
    }
  }
}
```

Then in any Claude Code session: ask "generate an image of …" — Claude will
discover and call `generate_image`.

## Cursor (stdio)

Add to `~/.cursor/mcp.json`:

```jsonc
{
  "mcpServers": {
    "tachyon": {
      "command": "/usr/local/bin/tachyon",
      "args": ["mcp", "serve"],
      "env": { "OPENAI_API_KEY": "sk-..." }
    }
  }
}
```

## Streamable HTTP (remote)

Server side:

```bash
TACHYON_MCP_TOKENS=$(openssl rand -hex 24) \
OPENAI_API_KEY=sk-... \
tachyon mcp serve \
  --transport http \
  --bind 0.0.0.0:7337 \
  --tokens "$TACHYON_MCP_TOKENS"
```

Client side (Cursor / Claude Code):

```jsonc
{
  "mcpServers": {
    "tachyon": {
      "url": "https://your-host:7337/mcp",
      "headers": { "Authorization": "Bearer <TACHYON_MCP_TOKENS>" }
    }
  }
}
```

## `?apikey=` URL query auth (non-spec extension)

For "share a one-shot URL" flows (e.g. tachyon admin console buttons), enable
the **non-spec** query-auth mode:

```bash
tachyon mcp serve --transport http \
  --tokens "$TACHYON_MCP_TOKENS" \
  --custom-query-auth
```

Issued URLs look like `https://host:7337/mcp?apikey=<token>`.

> **WARNING**. MCP spec 2025-06-18 §"Access Token Usage" explicitly
> requires that *"Access tokens MUST NOT be included in the URI query
> string"*. This mode is provided for tachyon-issued one-shot URLs only.
> Mitigations applied automatically:
>
> - `Referrer-Policy: no-referrer` on every response
> - `X-Content-Type-Options: nosniff`
> - Token masking in tracing logs (only first 4 + last 2 chars are logged)
> - 401 includes `WWW-Authenticate: Bearer realm="tachyon-mcp"`
>
> Pair with **short-lived tokens** (we recommend ≤ 15 min validity) and key
> rotation ≤ 90 days. Conformant MCP clients may reject query-auth URLs.

## Tool: `generate_image`

| Param | Type | Default | Notes |
|---|---|---|---|
| `prompt` | string | — | required |
| `model` | string | `gpt-image-2` | OpenAI model id |
| `size` | string | (provider default) | e.g. `1024x1024`, `1024x1536`, `auto` |
| `quality` | string | (provider default) | `low` / `medium` / `high` |
| `n` | u8 | `1` | clamped to 1–10 |
| `response_format` | string | `b64_json` | `b64_json` (returns MCP `image` content) or `url` (returns MCP `text` content) |
