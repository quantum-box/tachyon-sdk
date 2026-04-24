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

### OpenAI model 確定

- 一次情報: <https://platform.openai.com/docs/guides/image-generation>
- 一次情報: <https://platform.openai.com/docs/api-reference/images/create>
- 確定した model 名は実装時にここを更新する (現状 placeholder)

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

### HTTP/SSE 認証の primary-source 確認 ⚠ 要調査項目

MCP 仕様 (<https://modelcontextprotocol.io/specification/>) の **Authorization** セクションを
実装前に primary-source として確認し、以下を fix spec に残す:

1. URL query (`?apikey=...`) による認証は MCP 標準か / custom 拡張か
2. 標準が `Authorization: Bearer` のみの場合、query mode は custom extension として:
   - 非対応 MCP client では動作しない可能性
   - URL 共有時のログ/履歴漏洩リスク (管理画面発行 URL の `Referrer-Policy` 設定必須)
   - 互換性ノートを README / examples に明記
3. OAuth 2.0 Resource Server 部分を読み、apikey token の expiry / scope をどう伝えるかも確認

> この調査結果が出るまで、実装は `Authorization: Bearer` を first-class、
> `?apikey=` を opt-in の二次 path として切り出す方針にする。

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
- [ ] OpenAI docs で最新 GPT Image model 確定
- [ ] MCP spec で URL query 認証の標準性を primary-source 確認、fix spec に記載
- [ ] `rmcp` + 依存追加
- [ ] `mcp_cli.rs` / `mcp/` mod 実装 (stdio)
- [ ] HTTP+SSE transport 実装
- [ ] 認証 middleware (Bearer + query) + masking + referrer-policy
- [ ] key rotation / scope / expiry enforcement
- [ ] Claude Code PoC (stdio)
- [ ] Cursor PoC (HTTP+SSE)
- [ ] Cloud deploy path ドキュメント整備
- [ ] PR open & CI green
