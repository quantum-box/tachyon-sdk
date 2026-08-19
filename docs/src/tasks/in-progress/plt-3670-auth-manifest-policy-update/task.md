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
- 作成、更新、変更不要、失敗を区別できる human / JSON 出力にする。
- 既存 policy の action 追加・削除、pattern 追加・削除、description 更新を検証する
  回帰テストを追加する。
- PR CI の `cli` 向け `cargo test` 経路で回帰テストが実行されることを確認する。

## 非スコープ

- `tachyon-apps` 側の `sdk/cli` と `apps/tachyon-cli` の参照・変更。
- API contract や生成 SDK 自体の変更。
- `tachyon org policies get` が actions を `null` で返す問題の修正。
- CLI release/version bump。
- PR の merge。

## 完了条件

- [ ] 既存 policy が manifest の description、actions、action patterns と一致する。
- [ ] apply の出力が `updated` と理由付き `unchanged` を区別する。
- [ ] 更新 API が失敗した場合、apply は error を報告して非ゼロ終了する。
- [ ] 回帰テストが未修正コードで失敗し、修正後に成功する。
- [ ] `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` が成功する。
- [ ] feature branch の PR CI が green になる。

## 調査記録

- 2026-08-19: mirror issue と SSoT の現行実装を確認。policy apply は
  `POST /v1/auth/policies` の重複応答を `Skipped` に変換しており、反映確認なしで
  成功集計していた。
- 2026-08-19: `.github/workflows/ci.yml` が `cli` を working directory として
  `cargo test` を実行するため、CLI の unit / integration test は PR CI 経路に含まれる。

## 検証の境界

mock 回帰テストは、CLI が既存 policy に対して期待する PATCH request を送り、
API response に応じて `updated` / `unchanged` / `error` を正しく報告することを証明する。
実 API が request を永続化することまでは証明しない。利用可能な非本番環境で実 API
確認を行えない場合は、PR description と最終報告に未確認であることを明記する。
