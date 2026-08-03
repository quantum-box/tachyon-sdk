# PLT-2985: CLI profile ごとの PM デフォルト設定

Linear: https://linear.app/quantum-box/issue/PLT-2985

## 概要

Tachyon CLI の認証 profile ごとに、PM issue 操作の
`no_delegate` と `default_team` を保持する。認証 token を保存する
`profiles/<name>.json` はログインや refresh で再保存されるため、設定は
`~/.config/tachyon/settings.json` に分離する。

`pm issue`、top-level `issue`、互換入口の `linear issue` は同じ設定解決を
使う。既存の tenant IaC default は profile と環境変数のいずれにも値がない
場合の fallback として維持する。

## スコープ

- profile ごとの PM 設定の読み書き
- `tachyon config set|get|unset` コマンド
- CLI flag、環境変数、profile設定、既存既定の優先順位
- PM issue create/list/update への `default_team` 適用
- Linear issue create/update の auto delegation 既定への `no_delegate` 適用
- README と自動テストの更新

## 非スコープ

- Tachyon API / IaC の tenant default 設定変更
- 認証 profile JSON の形式変更
- PM provider API の変更
- 明示的に auto delegation を強制する新しい `--delegate` flag

## 対象

- `cli/src/settings.rs`
- `cli/src/settings_cli.rs`
- `cli/src/main.rs`
- `cli/src/pm_cli.rs`
- `cli/src/linear_cli.rs`
- `cli/tests/profile_settings.rs`
- `README.md`

## 実装

1. `settings.json` の型、load/save、profile設定解決を追加する。
2. `config set|get|unset` を追加し、`pm.no_delegate` と
   `pm.default_team` を型付きで操作する。
3. `TACHYON_PM_NO_DELEGATE` と `TACHYON_PM_DEFAULT_TEAM` を解決する。
4. 明示 `--team` / `--team-id` がない場合だけ既定teamを補完する。
5. 明示 `--delegate-id` を `no_delegate` より優先する。
6. 3つの issue CLI 入口で同じ挙動になることをテストする。

## 検証計画

- `cargo test --test profile_settings`
- `cargo test pm_cli::tests`
- `cargo fmt --check`
- 必要に応じて既存 `profile_auth` / `pm_issue_reconcile` を再実行する。

## 完了条件

- profile間で設定が分離され、認証JSONに設定が混入しない。
- 壊れた `settings.json` を黙って無視せずエラーにする。
- CLI、環境変数、profile、既存fallbackの優先順位をテストで確認する。
- 明示teamと明示delegateがprofile既定を上書きする。
- focused test と format check が成功する。

## リスクと後続

- `no_delegate=true` のprofileで明示的に auto delegation を強制する
  `--delegate` は別途検討する。
- SDKリリース後、利用側リポジトリで submodule pin の更新が必要になる。

## 実行ログ

- 2026-08-02: PLT-2985 と既存 profile / PM CLI / tenant IaC default を調査。
- 2026-08-02: taskdoc を作成し、実装開始。
- 2026-08-02: `settings.json`、`config set|get|unset`、PM設定解決を実装。
- 2026-08-02: `cargo test --test profile_settings` 成功（4 tests）。
- 2026-08-02: `cargo test --bin tachyon settings` 成功（5 tests）。
- 2026-08-02: `cargo test --bin tachyon pm_cli::tests` 成功（9 tests）。
- 2026-08-02: `cargo test --test profile_settings --test profile_auth --test
  pm_issue_reconcile -- --test-threads=1` 成功（4 + 11 + 8 tests）。並列実行では
  既存mock socketの `WouldBlock` によるflaky failureが2件出たが、各テスト単独と
  直列実行では成功した。
- 2026-08-02: `cargo fmt --check` 成功。
- 2026-08-02: 親worktreeの `clippy.toml` を読む通常の `cargo clippy -- -D
  warnings` はSDK既存の `reqwest::Client` 15箇所で失敗。SDK単独CI相当として
  空の `CLIPPY_CONF_DIR` を指定した同コマンドは成功した。
- 2026-08-03: `origin/main`（CLI v0.6.24）へrebaseし、PR用patch versionを
  v0.6.25へ更新。
- 2026-08-03: 最終品質チェック完了後、taskdocを`completed/v0.6.25`へ
  アーカイブ。
