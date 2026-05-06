# PLT-940 - tachyon CLI cloud app build watch

- Linear: PLT-940 (High)
- Branch: `feature/plt-940`
- Worktree: `~/tachyon-sdk.plt940`
- Status: In Review

## Goal

Claude Codeなどのコーディングエージェントが、Cloud App buildの完了待ちとログ確認を少ないコンテキストで実行できるCLIを追加する。

## Scope

- `tachyon compute builds watch` を追加し、最新buildまたは指定buildのログとステータスを完了まで監視する。
- buildが失敗・キャンセル・タイムアウトした場合は非0で終了する。
- `--agent` 出力モードを追加し、過剰なテーブル表示や重複ステータスを避けるJSON Linesを出す。
- 既存の `compute logs --follow` にも `--agent` を追加し、互換的に省出力ログを使えるようにする。
- READMEに利用例を追加する。

## Design

既存のCompute namespaceに合わせて、トップレベルではなく `compute builds watch` に追加する。

想定コマンド:

```sh
tachyon compute builds watch <app-id>
tachyon compute builds watch <app-id> --build-id <build-id>
tachyon compute builds watch --build-id <build-id> --agent
```

Agent modeはJSON Linesで以下を出す:

```json
{"type":"build","build_id":"bld_x","status":"running"}
{"type":"log","build_id":"bld_x","message":"..."}
{"type":"result","build_id":"bld_x","status":"succeeded","exit_code":0}
```

## Checklist

- [x] Issue/context確認
- [x] Worktree作成
- [x] CLI引数追加
- [x] watch実装
- [x] tests追加
- [x] README更新
- [x] fmt/test
- [x] Draft PR
- [x] LinearをIn Reviewへ更新

## Validation

- `cargo fmt` ✅
- `cargo test --test compute_watch --test reproduce` ✅
- `cargo test` ✅
- `cargo run --quiet -- compute builds watch --help` ✅
- `git diff --check` ✅

## Result

- Draft PR: https://github.com/quantum-box/tachyon-sdk/pull/77
- Linear: PLT-940 updated to In Review

## Notes

- `compute builds watch` は `--build-id` 指定だけでも動く。app指定がない場合はapp解決APIを呼ばない。
- `compute logs --follow --agent` でも同じJSON Linesのログ・結果を出す。
- Agent modeではログメッセージを500文字に丸め、ステータスは変化時と結果だけ出力する。
