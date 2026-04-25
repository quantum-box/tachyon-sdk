# PLT-835: tachyon CLI に MCP server mode と image generation tool を追加

## 目的

`tachyon` CLI (Rust) に **MCP (Model Context Protocol) server mode** を追加し、
Claude Code / Cursor などの MCP client から画像生成を呼び出せるようにする。

初期 tool として `generate_image` を expose し、内部は OpenAI 公式 API
(最新 GPT Image 系 model) を直接叩く。

## Scope (base dispatch + CEO 追加指示 2026-04-24 統合)

1. **transport 3 mode**: stdio / HTTP+SSE / Cloud 全部対応
   - `tachyon mcp serve` — stdio (local dev / Claude Code / Cursor)
   - `tachyon mcp serve --http <addr>` — HTTP(+SSE) transport
   - Cloud mode: tachyon-apps (Compute) への deploy を想定した常設 URL
2. MCP tool `generate_image`
   - input: `prompt`, `model?`, `size?`, `quality?`, `n?`, `response_format?`
   - output: MCP content block(s) — `image` (base64) または `text` (URL)
3. **認証の二系統**
   - `Authorization: Bearer <apikey>` header (RFC 6750 標準)
   - `?apikey=XXX` URL query (tachyon 管理画面 URL 発行ボタン用)
   - ただし後者は **MCP spec 上標準かどうか未確定** → 実装前に primary-source 確認必須
4. **セキュリティ緩和策 (MUST)**
   - key rotation 90 日 (発行時に `expires_at` を埋める / 期限切れで 401)
   - scope 限定 (tool 単位 ACL: 例 `generate_image` のみ許可)
   - log masking (header / query の apikey を `***` に置換)
   - Referrer-Policy: `no-referrer` 相当の HTTP response header
   - expiry 明示 (key metadata に `issued_at` / `expires_at`)
5. 環境変数
   - `TACHYON_API_KEY`: 既存基盤
   - `OPENAI_API_KEY`: 新規 (`generate_image` 用)
   - 未設定時は **tool registration を skip + stderr warn**
6. tachyon 管理画面で「MCP URL 発行ボタン」を押すと、scope/expires/apikey を埋めた
   一発 URL が出る — 将来の backend integration を見据えた設計
7. PoC: Claude Code + Cursor の 2 client で動作確認

## Non-goals (今 iteration)

- 管理画面 UI 実装自体 (別 task)
- `tachyon-apps` recycle pool 連携 (将来別 task、ただし Cloud mode は API 形状を揃える)
- Tachyon 既存 `/v1/images/generations` proxy との統合 (このルートとは独立)

## 設計方針

### Crate 選定

- Rust MCP SDK: [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) (official)
  - stdio + HTTP/SSE transport をサポート
  - `#[tool]` マクロで tool schema 自動生成

### CLI wiring

- `cli/src/main.rs` に `Mcp(mcp_cli::McpArgs)` を追加
- `cli/src/mcp_cli.rs` を新規作成
  - `McpCommand::Serve { transport, bind, ... }`
  - transport enum: `Stdio` (default), `Http { addr }`, `HttpSse { addr }`
  - `OPENAI_API_KEY` 未設定時は `generate_image` register を skip + warn

### OpenAI model 確定 (2026-04-24 primary source)

- **採用 model**: `gpt-image-2` (snapshot `gpt-image-2-2026-04-21`)
- endpoint: `POST /v1/images/generations`
- 出典:
  - <https://developers.openai.com/api/docs/models/gpt-image-2>
  - <https://openai.com/index/introducing-chatgpt-images-2-0/>
- 旧 `gpt-image-1` / `gpt-image-1.5` は tachyon CLI の既存 `image` subcommand でもデフォルトだが、
  MCP tool の default は最新 `gpt-image-2` を採用する
- サイズ / quality / response_format の詳細は model card に載っていない → 実装時に
  `/v1/images/generations` の API reference で確認 (platform.openai.com は 403 のため
  developers.openai.com/api を参照するか API error message で validate)

### MCP tool output 形式 (spec 準拠)

MCP `tools/call` response の content block:

```jsonc
// base64 mode
{
  "content": [
    { "type": "image", "data": "<base64>", "mimeType": "image/png" }
  ]
}

// url mode
{
  "content": [
    { "type": "text", "text": "https://..." }
  ]
}
```

複数枚の場合は content 配列に複数 block を積む。

### 認証未設定時の挙動

```rust
if std::env::var("OPENAI_API_KEY").ok().is_none() {
    eprintln!("[tachyon mcp] OPENAI_API_KEY not set — skipping generate_image tool registration");
    // generate_image は register しない
} else {
    // register generate_image
}
```

server 自体は起動し、MCP handshake と `tools/list` には応答する (空 list or 他 tool のみ)。

### HTTP/SSE 認証の primary-source 確認結果 ⚠ 重要

**primary source**: MCP spec 2025-06-18 Authorization
(<https://modelcontextprotocol.io/specification/2025-06-18/basic/authorization>)

#### 決定事項

| 項目 | spec 規定 | tachyon 実装方針 |
|---|---|---|
| credential 送信方法 | `Authorization: Bearer <token>` header **のみ** | 標準準拠 |
| URL query に token | **"MUST NOT be included in the URI query string"** (仕様原文) | **非標準 / 仕様違反** として扱う |
| stdio transport の auth | spec 対象外 (env から取得 SHOULD NOT follow) | env var で OK |
| token audience | RFC 8707 `resource` param MUST | 将来 OAuth 採用時に対応 |
| 401 response | `WWW-Authenticate` header MUST | 実装する |

#### `?apikey=XXX` URL query 認証について (CEO 要件の扱い)

**結論: MCP 標準では明示的に禁止 (MUST NOT)**。したがって実装する場合は
**custom non-spec extension** と明示し、以下のリスクを受容すること:

- spec 違反: 準拠 MCP client は拒否または警告する可能性
- token leak リスク:
  - server / proxy の access log に記録される (log masking 必須)
  - HTTP Referrer header 経由で第三者サイトに漏洩 (Referrer-Policy 必須)
  - ブラウザ履歴 / 共有 URL 経由の漏洩
  - CDN / analytics への記録
- audience binding (RFC 8707) との両立が難しい

#### 推奨実装方針 (fix spec)

1. **first-class**: `Authorization: Bearer <tachyon-issued-token>` — spec 準拠
2. **tachyon 管理画面発行 URL** は `--custom-query-auth` opt-in flag が立ったときだけ
   受け付ける。default off。
3. 管理画面 URL 発行時は:
   - `Referrer-Policy: no-referrer` を embed する HTML 側で設定
   - URL を一度使ったら自動で Bearer token に昇格 (query → header 変換 handshake) を案内
   - URL 有効期限を短く (e.g. 15 min) して rotation 前提
4. log masking は **両パターンとも必須** (tracing filter で `authorization` header と
   `apikey` query を `***` 置換)
5. README に spec 違反である旨と互換性注意を明記

> この方針で実装する。CEO 要件は満たしつつ、spec 違反の事実とリスクはドキュメントで
> 明確化する。

### セキュリティ緩和策 実装メモ

| 項目 | 実装方針 |
|---|---|
| key rotation 90d | 発行 meta (json) に `issued_at`, `expires_at`; middleware で expiry check |
| scope 限定 | tool 名 allowlist を token meta に含め、`tools/call` で拒否 |
| log masking | tracing filter: header `authorization`, query `apikey` を `***` にマスク |
| Referrer-Policy | HTTP response header `Referrer-Policy: no-referrer` |
| expiry 表示 | 401 response に `WWW-Authenticate: Bearer error="invalid_token", error_description="expired"` |

## 影響範囲

- `cli/Cargo.toml` — `rmcp`, `axum` (HTTP mode), `tracing` dep 追加
- `cli/src/main.rs` — subcommand 追加
- `cli/src/mcp_cli.rs` — 新規 (transport dispatch, tool registration)
- `cli/src/mcp/generate_image.rs` — 新規 (OpenAI client wrapper)
- `cli/src/mcp/auth.rs` — 新規 (bearer + query middleware, masking)
- `docs/src/tasks/in-progress/plt-835/task.md` — 本 doc
- `examples/mcp/` — Claude Code / Cursor 用 sample config (予定)

## PoC 手順

### stdio / Claude Code

```jsonc
{
  "mcpServers": {
    "tachyon": {
      "command": "/abs/path/to/tachyon",
      "args": ["mcp", "serve"],
      "env": { "OPENAI_API_KEY": "sk-..." }
    }
  }
}
```

### HTTP+SSE / Cursor (Bearer)

```jsonc
{
  "mcpServers": {
    "tachyon": {
      "url": "http://localhost:7337/sse",
      "headers": { "Authorization": "Bearer <apikey>" }
    }
  }
}
```

### HTTP / ?apikey (管理画面発行 URL 想定) ※ MCP spec 標準性確認後

```
http://host:7337/sse?apikey=<issued-token>
```

## チェックリスト

- [x] taskdoc 初回 commit (CEO 追加指示反映版)
- [x] OpenAI docs で最新 GPT Image model 確定 → `gpt-image-2`
- [x] MCP spec で URL query 認証の標準性を primary-source 確認 → **spec で MUST NOT 禁止**
- [x] custom query-auth extension の opt-in flag (`--custom-query-auth`, default off)
- [x] `rmcp` 1.5 + 依存追加
- [x] `mcp/{server,openai,auth}.rs` + `mcp_cli.rs` 実装 (stdio + Streamable HTTP)
- [x] OPENAI_API_KEY 欠落時の tool registration skip + warn
- [x] Bearer auth + log masking + Referrer-Policy + WWW-Authenticate
- [x] `cargo build` green
- [x] stdio handshake / `tools/list` smoke test (with & without OPENAI key)
- [x] HTTP transport 401/200 smoke test (Bearer + custom query auth)
- [x] Claude Code 登録 + `tools/list` で `generate_image` 表示確認
- [ ] **要 real OPENAI key**: end-to-end image generation (本 task の PoC 完了条件)
- [ ] Cursor PoC (config sample 提供済 / 実機検証は別環境)
- [ ] PR open & CI green

## 実装サマリ

| ファイル | 役割 |
|---|---|
| `cli/src/mcp_cli.rs` | clap subcommand, transport dispatch, tracing init |
| `cli/src/mcp/server.rs` | `TachyonMcpServer` + `#[tool]` `generate_image` |
| `cli/src/mcp/openai.rs` | OpenAI `/v1/images/generations` クライアント (default `gpt-image-2`) |
| `cli/src/mcp/auth.rs` | Bearer + opt-in query auth + masking + Referrer-Policy |
| `examples/mcp/{README,claude-code,cursor}.{md,json}` | sample configs |

## 検証ログ抜粋

stdio without OPENAI_API_KEY → `tools/list` empty + stderr warn:

```
[tachyon mcp] OPENAI_API_KEY not set — skipping `generate_image` tool registration
{"jsonrpc":"2.0","id":2,"result":{"tools":[]}}
```

HTTP transport with `--tokens` (no `--custom-query-auth`):

```
no auth          → 401
Bearer t1        → 200
?apikey=t1       → 401  (spec compliant)
Response headers → Referrer-Policy: no-referrer, X-Content-Type-Options: nosniff
```

HTTP transport with `--custom-query-auth`:

```
?apikey=t1       → 200  (opt-in extension)
?apikey=bad      → 401  (token=*** masked in log)
```

Claude Code (haiku) sub-process visibility check:

```
$ claude -p --model haiku "List … tachyon-plt835 server."
1. generate_image — Generate an image from a text prompt using OpenAI's GPT Image model
```
- [ ] `rmcp` + 依存追加
- [ ] `mcp_cli.rs` / `mcp/` mod 実装 (stdio)
- [ ] HTTP+SSE transport 実装
- [ ] 認証 middleware (Bearer + query) + masking + referrer-policy
- [ ] key rotation / scope / expiry enforcement
- [ ] Claude Code PoC (stdio)
- [ ] Cursor PoC (HTTP+SSE)
- [ ] Cloud deploy path ドキュメント整備
- [ ] PR open & CI green
