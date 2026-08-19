# PLT-3670: auth manifest apply で既存 policy を更新する

Linear: https://linear.app/quantum-box/issue/PLT-3670  
Mirror: https://github.com/quantum-box/tachyon-apps/issues/8559

## 概要

`tachyon auth manifest apply` が既存 policy に対して作成 API だけを呼び、
重複応答を `skipped` として成功扱いするため、manifest の description、actions、
action patterns が反映されない silent success を修正する。

## スコープ

- `cli/src/commands/auth/manifest.rs` の policy apply を宣言的な更新にする。
- 生成済み SDK の `update_policy` を使用し、既存 policy の差分を PATCH する。
- 作成、更新、失敗を区別できる human / JSON 出力にする。現 API では既存
  membership を観測できないため、既存 policy を `unchanged` とは判定しない。
- 既存 policy の action / pattern の add/upsert、description 更新を検証する
  回帰テストを追加する。
- API の read contract に membership がないため stale action / pattern の削除と
  宣言との完全一致確認を行えないことを、apply の実行結果に毎回明示する。
- PR CI の `cli` 向け `cargo test` 経路で回帰テストが実行されることを確認する。

## 非スコープ

- `tachyon-apps` 側の `sdk/cli` と `apps/tachyon-cli` の参照・変更。
- API contract や生成 SDK 自体の変更。
- `tachyon org policies get` が actions を `null` で返す問題の修正。
- CLI release/version bump。
- PR の merge。

## 完了条件

- [x] 既存 policy に manifest の description、actions、action patterns を PATCH する。
- [x] 既存 policy の PATCH 成功を `updated` とし、未観測を `unchanged` としない。
- [x] stale membership を削除せず完全一致を検証できない制限を実行結果に出す。
- [x] 更新 API が失敗した場合、apply は error を報告して非ゼロ終了する。
- [x] 回帰テストが旧 POST/409 skip 経路を拒否し、修正後の PATCH 経路で成功する。
- [x] `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` が成功する。
- [ ] feature branch の PR CI が green になる。

## 調査記録

- 2026-08-19: mirror issue と SSoT の現行実装を確認。policy apply は
  `POST /v1/auth/policies` の重複応答を `Skipped` に変換しており、反映確認なしで
  成功集計していた。
- 2026-08-19: `.github/workflows/ci.yml` が `cli` を working directory として
  `cargo test` を実行するため、CLI の unit / integration test は PR CI 経路に含まれる。

## 検証の境界

mock 回帰テストは、CLI が既存 policy に対して期待する PATCH request を送り、
API response に応じて `updated` / `error` を正しく報告することを証明する。
実 API が request を永続化することまでは証明しない。利用可能な非本番環境で実 API
確認を行えない場合は、PR description と最終報告に未確認であることを明記する。

現 API の `PolicyResponse` は actions / action patterns を返さない。したがって、
`actionsToRemove` / `actionPatternsToRemove` を推測で送らず、declared membership の
add/upsert に限定する。stale membership の除去、宣言との完全一致、真の no-op 判定は
API read contract の follow-up 後に行う。

## 実行ログ

- 2026-08-19: API repository の `save_policy_actions` と
  `save_policy_action_patterns` がともに duplicate key 時に effect を更新する upsert
  であることを確認。
- 2026-08-19: `cargo +stable fmt --check` 成功。
- 2026-08-19: `cargo +stable clippy -- -D warnings` 成功。
- 2026-08-19: `cargo +stable test` 成功。unit 344件と全 integration targets が成功し、
  `auth_manifest_auth` 14件に追加した PATCH success / human output / error tests を含む。
- 実 API への適用・永続化確認は未実施。mock は CLI request と result handling のみを
  証明し、server persistence、stale membership 除去、完全収束は証明しない。
