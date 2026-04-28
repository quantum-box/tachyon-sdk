# PLT-958 — tachyon-cli `image generate` 参考画像 input + side finding (default model gpt-image-2 統合)

> **role**: leader-plt958 (claude account2)
> **primary cwd**: `~/tachyon-sdk.leader-plt958` (this repo, branch `feature/plt-958-cli-reference-image`, origin `quantum-box/tachyon-sdk`)
> **secondary cwd**: `~/tachyon-apps.plt816` (branch `feature/plt-958-cli-reference-image-api`, origin `quantum-box/tachyon-apps`)
> **directive anchor**: CEO 2026-04-28 17:59 JST 直命 / Linear PLT-958 / scoping commit `7b1a757`
> **scoping SSoT**: `~/knowledge/prompts/pdm-platform/state/plt-958-cli-reference-image/scoping.md` (Section 5–9 = 実装仕様)

## 1. 担当 scope

`tachyon-sdk` 側 (CLI) のすべての改修と、`tachyon-apps` 側 (api/provider/usecase/domain) のすべての改修を 1 leader (本 leader) で完遂する。

### 完了条件

| # | 完了条件 | 状態 |
|---|---|---|
| 1 | `tachyon-sdk` PR (`feat(plt-958): ...`) merged on green CI | pending |
| 2 | `tachyon-apps` PR (`feat(plt-958): ...`) merged on green CI | pending |
| 3 | Lambda deploy 後 prod E2E 3 cases (1 ref / 2 refs / no ref) 全 200 OK + 画像 evidence | pending |

## 2. SSoT 確定 (CLAUDE.md L562-595)

- **CLI**: `quantum-box/tachyon-sdk` の `cli/src/image_cli.rs` のみ
- **凍結**: `tachyon-apps/apps/tachyon-cli` (subtree split 残骸、触らない)
- **API/Provider**: `quantum-box/tachyon-apps` の以下:
  - `packages/llms/src/adapter/axum/image_generation_handler.rs`
  - `packages/llms/src/usecase/generate_image.rs`
  - `packages/llms/domain/src/image_generation.rs`
  - `packages/providers/openai/src/image.rs`

## 3. データフロー (確定)

```
CLI (file path)
  → (read & validate magic byte + size <25MB)
  → (base64 encode each)
  → JSON: { ..., reference_images: ["<b64>", ...] }
  → POST /v1/images/generations (Lambda)

API handler (Json) → InputData → usecase (validate) → domain Request → OpenAI provider
  → (decode base64 → multipart/form-data: image[]=<bytes>;filename="ref0.png")
  → POST https://api.openai.com/v1/images/generations
```

CLI ↔ API は JSON のみ (multipart 化しない)。multipart は **provider ↔ OpenAI 間** のみ。

## 4. 段階タスク (scoping Section 7)

| ID | 作業 | repo | 工数 |
|---|---|---|---|
| B-1 | API handler `ImageGenerationRequest.reference_images: Option<Vec<String>>` 追加 | apps | 2h |
| B-2 | OpenAI provider multipart 対応 (`reqwest::multipart::Form`)、Cargo.toml `multipart` feature | apps | 3h |
| B-3 | usecase + domain `ImageGenerationRequest` 同 field 追加 + validation + unit test | apps | 2h |
| B-4 | CLI `--reference-image` 追加 + magic byte / size validation + base64 + side finding (default gpt-image-2) | sdk | 2h |
| B-5 | scenario test (apps) + integration test (cli) | both | 2h |
| B-6 | PR 起票・merge (apps → sdk 順) + Lambda deploy + prod E2E | both | 1h + E2E |
|   | **合計** |   | **12h** |

## 5. 既存実装 verify (Explore + leader 確認済)

### CLI (`cli/src/image_cli.rs`)
- `Generate::model` の `default_value = "gpt-image-1.5"` (L32) → side finding で `gpt-image-2` に変更
- request: JSON `GenerateImageRequest` (L61-71) → `reference_images: Vec<String>` 追加
- `base64 = "0.22"` 既に dependency
- `reqwest` features: `["json", "form", "query", "rustls"]` (multipart 不要、CLI は base64 JSON のみ)

### API handler (`packages/llms/src/adapter/axum/image_generation_handler.rs`)
- `ImageGenerationRequest` (L19-51) に `reference_images: Option<Vec<String>>` 追加
- `generate_image()` (L123-148) で InputData 構築時に passthrough

### usecase (`packages/llms/src/usecase/generate_image.rs`)
- `GenerateImageInputData` (L26-55) に `reference_images: Option<Vec<String>>` 追加
- `build_request()` (L251-401) で:
  - reference_images 指定時 model != gpt-image-2 → 400 reject
  - count > 16 → 400 reject
  - 各要素 base64 decode 試行 → 失敗で 400
  - decoded size 25MB 超 → 400 reject
  - magic byte (PNG/JPEG) check → mismatch で 400

### domain (`packages/llms/domain/src/image_generation.rs`)
- `ImageGenerationRequest` (L132-162) に `reference_images: Option<Vec<Vec<u8>>>` 追加 (decoded bytes、provider 層で multipart に流す)

### OpenAI provider (`packages/providers/openai/src/image.rs`)
- `send_image_request()` (L78-109) を分岐:
  - reference_images None → 既存 JSON path 維持
  - reference_images Some → multipart/form-data path
    - `Form::new().text("prompt", ...).text("model", "gpt-image-2")...`
    - 各 image: `Part::bytes(bytes).file_name(format!("ref{i}.png")).mime_str("image/png")?`
    - `image[]` field name (OpenAI 仕様)
- `Cargo.toml` に reqwest `multipart` feature 追加

## 6. Risk / 注意

- **reqwest multipart feature**: openai package `Cargo.toml` で features override しているので、`"multipart"` を明示追加必須
- **base64 encoding 4/3x overhead**: 16 枚 × 25MB = 400MB raw → ~533MB base64。Lambda payload 上限注意 (API Gateway 10MB / Lambda direct 6MB sync invoke)。**実用上は数枚 × 数 MB の想定**、16 枚 25MB upper bound は warning に留める
- **PLT-955 hotfix (response_format omit) との co-exist**: 既に provider 側で `response_format: None` に固定済 → reference_images path でも維持
- **OpenAI form field 名**: `image[]` (複数) または `image` (単数) — 公式 doc 仕様に従う、scoping Section 4 primary-source verify 済
- **CEO mandate side finding**: default model gpt-image-2 化は同 PR 統合 (CLAUDE.md L420-440)

## 7. 進捗ログ

- 2026-04-28 18:30 JST: leader-plt958 起動、scoping FINAL Read、両 repo branch verify、taskdoc 起票
- (以降 commit 単位で追記)

## 8. 報告

- 6h ごと: PdM-Platform (window 17 / pdm-pf) に 進捗報告
- 完了時: 両 PR merged + prod E2E PASS evidence + Linear PLT-958 Done flip 依頼
- self-kill: `tmux kill-window -t work:3` (数値 index、display-message 禁止、precedent: leader-sol206 誤 kill 事故)
