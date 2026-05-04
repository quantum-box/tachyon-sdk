PLT-1098 tachyon-cli init + repo 内 default config 読取 (CEO 5/4 directive、leader-plt1098-codex)

## あなたの役割
leader-plt1098-codex (work:N、tachyon-sdk.leader-plt835 = feature/plt-1098、HEAD = origin/main dd50b56 派生)

## CEO directive (5/4 via COO)
Linear PLT-1098: https://linear.app/quantum-box/issue/PLT-1098
title: tachyon-cli init で tachyon.yml 雛形生成 + repo 内実行時 default config 読み取り
priority: High、PdM label

## scope (2 機能)

### 1. `tachyon init` subcommand (新規)
- repo root で実行 → `tachyon.yml` 最小テンプレ生成
- **interactive prompt**:
  - app name (default = repo dir name)
  - framework (selectable: `nextjs` / `vite` / `static` / `none`)
  - tenant_id (required、未入力なら error)
- **flag override**:
  - `--name <NAME>`
  - `--framework <FRAMEWORK>`
  - `--tenant-id <ID>`
  - `--non-interactive` (prompt skip、未指定 flag は default)
  - `--force` (既存 tachyon.yml 上書き、未指定なら exists error)
- 生成テンプレ最小例:
  ```yaml
  apiVersion: tachyon/v1
  kind: CloudApp
  metadata:
    name: <name>
    tenant_id: <tenant_id>
  spec:
    framework: <framework>
    # build_command: yarn build
    # output_directory: dist
  ```

### 2. repo 内実行時 default config 自動読取
- subcommand (compute / builds / cloud-apps 等) で **default 引数として埋まる**
- **探索ロジック**: cwd → 上位 dir を git root まで遡及して `tachyon.yml` を探す
- **優先順位**:
  1. `TACHYON_CONFIG` env (絶対 path or 相対 path)
  2. `--config <PATH>` flag (CLI 引数)
  3. cwd / 上位 dir 探索 (git root まで)
- 見つかった config の `metadata.name` (app name) / `metadata.tenant_id` 等を default に投入
- 既存 explicit flag (`--app-id` 等) があれば config より優先
- config 不在時は従来通り (エラーにしない)

## SSoT
- repo: `quantum-box/tachyon-sdk` (このリポジトリ、worktree = leader-plt835 を recycle)
- cli code: `cli/` (Rust)
- kb: `~/knowledge/src/projects/tachyon-sdk/overview.md` (storekit SSoT + tachyon-cli SSoT)

## 実装手順

1. **taskdoc commit (CLAUDE.md L1276-1287 必須)**:
   - `docs/src/tasks/in-progress/plt-1098/task.md` を feature branch に commit
   - tachyon-sdk repo に docs/src/tasks/ が無い場合は state/plt-1098/ で代替 OK

2. **既存 cli/ 構成確認**:
   - `cli/Cargo.toml` で binary name / dependencies 把握
   - `cli/src/main.rs` で subcommand dispatch pattern 把握 (clap derive 想定)
   - 既存 subcommand (compute / builds / cloud-apps) で config / flag 解析 pattern 把握

3. **`tachyon init` subcommand 実装**:
   - `cli/src/commands/init.rs` 新規 module
   - clap derive で interactive prompt + flag override
   - dialoguer / inquire crate 等で interactive UI (既存 dependency 確認)
   - tachyon.yml 生成 = serde_yaml で template struct serialize
   - `--force` 不在時 既存 file 検知 → exit code 1 + 明示 error message

4. **default config 自動読取 実装**:
   - `cli/src/config/loader.rs` 新規 helper
   - 探索 fn: cwd → parent → ... → git root (.git/ の存在で判定 or `gix` crate)
   - 優先順位 cascade: env (`TACHYON_CONFIG`) → `--config` flag → 探索 fallback
   - 既存 subcommand (compute / builds 等) の引数解析に default injection
   - 既存 explicit flag があれば config 値を上書きしない

5. **unit test 追加**:
   - `tachyon init` non-interactive flag 経路 (各 framework 値、tenant_id required、force 動作)
   - tachyon init 既存 file あり + --force なし → error
   - 自動読取 探索ロジック (cwd / parent / git-root に file 配置 fixture)
   - 優先順位 (env > flag > 探索) 検証
   - 既存 explicit flag が config を上書きすること verify

6. **integration test (CLI 全体)**:
   - tempdir で git init + tachyon init non-interactive 実行 → tachyon.yml 生成 verify
   - tachyon init --force で既存 file 上書き verify
   - repo 内 cd subdir で tachyon compute list (or 任意) 実行時 config 自動読取 verify

7. **PR 作成 (target = main)**:
   - title: `feat(cli): tachyon init + repo config auto-discovery (PLT-1098)`
   - description: 2 機能仕様 + 探索ロジック + 優先順位 + verify 手順
   - 「完了報告は PdM-PF (work:17) に 1 行 push」を明記

8. **CI green wait**: cargo test / clippy / fmt / rustfmt 全 pass

9. **AI review clear → 即 admin merge** (CLAUDE.md release-policy.md L1132 ルール、CEO 確認不要):
   ```bash
   gh pr merge <PR#> --merge --admin --delete-branch
   ```
   - 例外: 本番 DML / breaking change の場合は HOLD
   - tachyon-cli は public crate / npm package の可能性あり、breaking change 判定は慎重に (新 subcommand 追加は non-breaking、config auto-load も既存挙動を破壊しないなら non-breaking)

10. **整合性 verify**:
    - **実 repo で tachyon init 動作確認**:
      - 新規 tempdir で `tachyon init --non-interactive --name plt1098-verify --framework vite --tenant-id test-tenant` → tachyon.yml 生成
      - 既存 file あり + --force なし → error 確認
    - **repo 内 tachyon コマンド動作確認**:
      - 上記 tempdir で `tachyon compute list` (or 既存 subcommand) 実行 → config 自動読取で tenant_id 等が default 引数に埋まる verify
      - cwd を subdir に移動しても探索が git root まで遡って正しく config 取得 verify

11. **完遂後**:
    - taskdoc を `docs/src/tasks/in-progress/plt-1098/task.md` (or state/plt-1098/) から `docs/src/tasks/completed/plt-1098/` (or state/plt-1098/completed/) へ移動 + commit
    - Linear PLT-1098 Done flip COO (work:0 = coo:0) 経由依頼 (PdM-PF Linear MCP 不可)
    - PdM-PF (work:17) に 1 行 ack 報告
    - Docker cleanup: `docker volume prune -f && docker image prune -f`
    - self-kill: `tmux kill-window -t work:N` (数値 index 直接指定)

## 絶対禁止事項

- main 直 push (CLAUDE.md L1217-1224、2026-04-18 leader-404fix 本番破損事例)
- expanded credential 形式 commit (env file 全文 / SQL UPDATE 全文 / SA secret env 全文 / API token 平文)
- self-kill で `tmux display-message -p '#W'` 等 active window 解決禁止 → 必ず数値 index `work:N` を直接指定
- claude-review FAILURE が本物の review 指摘の場合、admin merge 強行禁止、修正 PR 必須
- claude (account1/account2) で新規 spawn 起動禁止 (5/5 7am まで rate limit) → spawn 必要時は codex で起動
- breaking change 入れる場合は CEO 事前確認必須 (config auto-load の既存挙動破壊チェック含む)

## 起動アカウント routing

- 自分 (leader-plt1098-codex) = codex 起動 (`codex --dangerously-bypass-approvals-and-sandbox`)
- 配下 spawn / QA = codex 起動 (claude 禁止、ac1/ac2 rate limit 中)
- worktree = `~/tachyon-sdk.leader-plt835` (recycle、現在は feature/plt-1098)

## 報告 milestone (各 1 行で work:17 = PdM-PF に push)

- 起動完了 + branch HEAD verify 1 行 ack
- 既存 cli/ 構成把握 + 実装 plan 1 行
- PR open + CI green 待ち 1 行 (PR URL)
- claude-review pass + admin merge 完了 1 行 (PR URL + merge SHA)
- 実 repo verify (tachyon init + repo 内 config 自動読取) PASS 1 行
- Linear Done flip 完了 + self-kill 1 行
- それ以外 narration 不要

## ack template
起動完了で PdM-PF (work:17) に 1 行 ack 返却:
```
tmux send-keys -t work:17 -l 'leader-plt1098-codex 起動完了 (work:N、tachyon-sdk.leader-plt835 = feature/plt-1098、HEAD dd50b56 派生、CEO 5/4 directive)' && sleep 0.3 && tmux send-keys -t work:17 Enter
```

GO 即着手。
