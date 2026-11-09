# PLT-3071: Sentry issue latest event JSON contract

Linear: https://linear.app/quantum-box/issue/PLT-3071

## 概要

Tachyon API が `issues view` の `latest_event` に返す event ID、timestamp、
level、message、調査用 authz tags を CLI の JSON 再 serialize 後も保持する。
現行 `SentryEventResponse` が持たない `level` と `tags` が落ちる不具合を修正する。

## スコープ

- `SentryEventResponse` の provider alias、level、typed tag map
- `tachyon ops sentry issues view <issue_id> --json` の request path / stdout JSON test
- unknown/raw event field と synthetic secret sentinel の非出力 test

## 非スコープ

- Sentry write command / scope の変更
- access token、Authorization header、DSN、request payload、secret 値の出力
- human-readable output の変更
- API rollout 前の CLI release

## 実装・検証

- [x] `eventID` / `dateCreated` aliases、`level`、`BTreeMap<String, String>` tags を追加。
- [x] deserialize -> serialize で固定 latest-event contract が残る unit test を追加。
- [x] `issues view --json` の API path、8 tags、raw field非出力の integration test を追加。
- [x] `cargo fmt --check`
- [x] `cargo clippy -- -D warnings`
- [x] `cargo test`
- [ ] apps API PR の rollout 依存を SDK PR に明記。

## リリース順序

tachyon-apps 側の read-only API 変更を先に rollout し、その後に SDK 側の CLI を
release する。feature PR で CLI version を上げず、現行 repository 運用どおり
release commit と分離する。

## 実行ログ

- 2026-08-04: `cargo +stable fmt --manifest-path cli/Cargo.toml --check` 成功。
- 2026-08-04: `cargo +stable clippy --manifest-path cli/Cargo.toml -- -D warnings` 成功。
- 2026-08-04: `cargo +stable test --manifest-path cli/Cargo.toml` 成功。unit 299件と全 integration targets。
- 2026-08-04: `sentry_cli` 3件成功。view JSON path / fields / 8 tags / unknown field非出力を確認。
