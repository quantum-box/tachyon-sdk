PLT-1099 tachyon.yml auth 定義 + tachyon auth CLI clientid/secret 発行 (CEO 5/4 directive、leader-plt1099-adr-codex)

## あなたの役割
leader-plt1099-adr-codex (work:N、tachyon-sdk.leader-plt724 = feature/plt-1099-auth-adr、HEAD = origin/main 21a29a7 派生)

## CEO directive (5/4 14:10 JST via COO)
Linear PLT-1099: https://linear.app/quantum-box/issue/PLT-1099
CEO 原文: 「tachyon.yml に認証を使うって定義して clientid, secret を発行するようにしたい」
PLT-1097 / PLT-1098 と同じ Tachyon Cloud Apps DX 連続施策ライン。

## 段階 (sequential 3 フェーズ、本 leader は Phase 1 まで)

### Phase 1 (本 leader 担当) — ADR 起票 + COO/CEO 確認
1. **ADR sub-issue 起票** (Linear で PLT-1099 の sub-issue として「PLT-1099-ADR」起票、PdM-PF MCP 不可 → 内容ドラフトを PdM-PF (work:17) 経由で COO に依頼)
2. ADR 内容で COO/CEO 確認待ち
3. ADR FIX 通知を受領

### Phase 2 (別 leader、Phase 1 完了後 PdM-PF が起動)
- ADR FIX 後 implementation leader を別 worktree で起動
- 本 leader は Phase 1 完遂で self-kill

### Phase 3 (Phase 2 leader 担当)
- PR 起票 → CI green → admin merge → CLI release

## ADR スコープ (5 章必須)

### 章 1: tachyon.yml auth schema 提案
- `auth:` block を `tachyon.yml` トップレベルに追加
- `providers[]` 配列で複数 provider 対応 (default = 1 provider)
- 各 provider entry の必須/optional fields:
  - `name` (string、provider 識別子)
  - `type` (enum: `oauth2_client_credentials` / `api_key` / `service_account` / 他)
  - `audience` (string、token issuance scope、optional)
  - `expiry_days` (int、credential 有効期限、default = 90)
  - その他必要 fields
- 例 yaml snippet 2-3 つ提示 (single provider / multi provider / minimal)

### 章 2: 発行先 4 option 比較
比較項目: security / latency / cost / migration effort / vendor lock-in / operational complexity の 6 項目
- (a) **AWS Secrets Manager** (ASM): 既存 tachyon infra で広く使用 (PLT-1058/1079/1056 等)、tf 化済 / kms 暗号化
- (b) **Cloudflare Workers Analytics Engine (WAE)** or CF Secrets Store: Cloudflare 1st party、Worker 直アクセス
- (c) **Tachyon credential service** (新規): bakuure-api or 専用 microservice、自社 SSoT
- (d) **ローカル `.tachyon/credentials.json` chmod 600**: gitignore + perms 600、開発者ローカルのみ (本番不可、bootstrap 用)

各 option の適用 use case (dev / staging / production) を明示。

### 章 3: rotate API 設計
- `tachyon auth rotate <provider>` CLI で credential 失効 + 新規発行
- 旧 credential の grace period (zero-downtime rotation)
- 監査 log 出力 (どの SA / when / by whom)
- ASM 採用時 = AWS SDK で `UpdateSecret` + `PutSecretValue`
- CF WAE / Tachyon credential service 採用時 = 各 API spec
- error handling (rotate 中に失敗 → rollback)

### 章 4: 既存資源衝突挙動
- 既に同 name の provider が tachyon.yml に存在 → error / overwrite / merge どれを default にするか
- 既存 ASM secret / CF Secret 名 collision 時の挙動 (`--force` flag 必要か)
- `tachyon auth init` (新規) と既存 secret の関係
- multi-environment (dev/staging/prod) で同 provider name 違う credential 値の扱い

### 章 5: GO/NO-GO 推奨
- 4 発行先の中で **PdM-PF 推奨 1 つ** + 根拠 3 行
- Phase 0/1/2 段階導入提案 (例: Phase 0 = local credentials.json から、Phase 1 = ASM 統合、Phase 2 = full rotate API)
- 各 phase の deliverable + 完遂指標

## 一次情報

- 既存 tachyon-sdk cli/: `cli/src/auth.rs` / `cli/src/config/` (PLT-1098 で導入された config loader)
- tachyon-apps cluster/n1-aws/: 既存 ASM secret pattern (read のみ、書き換え禁止)
- knowledge: `~/knowledge/src/projects/tachyon-sdk/overview.md` (storekit + tachyon-cli SSoT)
- AWS Secrets Manager docs: https://docs.aws.amazon.com/secretsmanager/
- Cloudflare Secrets Store docs: https://developers.cloudflare.com/

引用は URL + 取得日 (2026-05-04) 併記。

## 実装手順 (本 leader = Phase 1)

1. **taskdoc commit** (CLAUDE.md L1276-1287 必須):
   - `state/plt-1099/task.md` に Phase 1 ADR 起票進捗記録 (in-progress)
   - tachyon-sdk repo は `docs/src/tasks/` 不使用、state/ 配下で OK

2. **既存 cli/src/auth.rs 確認**:
   - 現状 auth subcommand があるなら schema 拡張提案ベース確認
   - `cli/src/config/` で PLT-1098 の config loader pattern 確認

3. **既存 tachyon-apps secret pattern 確認** (read のみ):
   - cluster/n1-aws/ の ASM secret terraform location 特定 (e.g. PLT-1056 storage SA、PLT-1079 OAuth2)
   - 命名規約 + tag pattern 引用

4. **ADR 執筆** (`state/plt-1099/adr-tachyon-yml-auth.md`):
   - 5 章 (schema / 発行先 4 option 比較 / rotate API / 既存資源衝突 / GO/NO-GO 推奨)
   - 例 yaml snippet 提示
   - secret material は **絶対書かない** (clientid/secret 等の実値は ADR にも commit にも書かない)

5. **PR 作成 (target = main)**:
   - title: `docs(plt-1099): ADR draft for tachyon.yml auth + clientid/secret issuance`
   - description: ADR 5 章 概要 + COO/CEO 確認依頼
   - PR open 後 PdM-PF (work:17) に PR URL 1 行報告

6. **PdM-PF (work:17) 経由 COO に Linear sub-issue 起票依頼**:
   - sub-issue title: `PLT-1099-ADR (sub): tachyon.yml auth schema + 発行先 4 option ADR`
   - parent issue: PLT-1099
   - description ドラフトを PdM-PF (work:17) に push、PdM-PF が COO 経由で Linear に投入

7. **COO/CEO 確認待ち**:
   - ADR PR は AI review pass しても **HOLD** (CEO 確認待ち、自動 admin merge 禁止)
   - COO から「ADR FIX、Phase 2 GO」通知を待つ

8. **Phase 1 完遂後**:
   - taskdoc を `state/plt-1099/task.md` (in-progress) → `state/plt-1099/task-phase1-completed.md` 等に移動 + commit
   - PdM-PF (work:17) に Phase 1 完遂 ack 報告
   - self-kill: `tmux kill-window -t work:N` (数値 index 直接指定)

## 絶対禁止事項

- **secret material (clientid/secret/access_token/refresh_token 等) を ADR / taskdoc / commit / PR description / Linear comment に書かない** (CLAUDE.md L1383-1408 expanded credential 禁止、PR #3509 P0 incident)
  - 良い例: `client_secret: <REDACTED>` / `client_secret: <stored in ASM tn_<env>/auth/<provider>/secret>`
  - 悪い例: `client_secret: pk_<base64>` 実値、env file 全文、SQL UPDATE 全文
- main 直 push (CLAUDE.md L1217-1224、2026-04-18 leader-404fix 本番破損事例)
- self-kill で `tmux display-message -p '#W'` 等 active window 解決禁止 → 必ず数値 index `work:N` を直接指定
- claude (account1/account2) で新規 spawn 起動禁止 (5/5 7am まで rate limit) → spawn 必要時は codex で起動
- ADR PR を勝手に admin merge (CEO 確認必須、AI review clear でも HOLD)
- 個人名・肩書 完全排除 (ADR 内で引用しない、TWS brand policy 同等)

## 起動アカウント routing

- 自分 (leader-plt1099-adr-codex) = codex 起動 (`codex --dangerously-bypass-approvals-and-sandbox`)
- 配下 spawn / QA = codex 起動 (claude 禁止、ac1/ac2 rate limit 中)
- worktree = `~/tachyon-sdk.leader-plt724` (recycle、現在 feature/plt-1099-auth-adr)

## 報告 milestone (各 1 行で work:17 = PdM-PF に push)

- 起動完了 + branch HEAD verify 1 行 ack
- 既存 cli/src/auth.rs + config/ 把握 + ADR 執筆 plan 1 行
- ADR 章 1-5 draft 完了 1 行 (file path 含む)
- PR open + sub-issue 起票依頼 1 行 (PR URL + sub-issue title)
- COO/CEO 確認待ち 1 行 (HOLD 中)
- Phase 1 完遂 + self-kill 1 行
- それ以外 narration 不要

## ack template
起動完了で PdM-PF (work:17) に 1 行 ack 返却:
```
tmux send-keys -t work:17 -l 'leader-plt1099-adr-codex 起動完了 (work:N、tachyon-sdk.leader-plt724 = feature/plt-1099-auth-adr、HEAD 21a29a7 派生、CEO 5/4 directive、Phase 1 = ADR draft only)' && sleep 0.3 && tmux send-keys -t work:17 Enter
```

GO 即着手。
