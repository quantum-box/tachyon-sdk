# PLT-923: tachyon CLI self-update が `Unexpected tag format` で失敗

Linear: https://linear.app/quantum-box/issue/PLT-923
Priority: Urgent / Status: In Progress

## 症状

```
$ tachyon self-update
Error: Unexpected tag format: v0.71.1 (expected prefix 'tachyon-cli-v')
```

tachyon-cli ユーザー全員 self-update 不可 = blocker。

## Root cause (primary-source verified)

ユーザーが install 済の `tachyon` バイナリ (v0.5.0) は、freeze 済の `tachyon-apps`
リポジトリでビルドされた旧 self-update 実装に依存している:

- `tachyon-apps/apps/tachyon-cli/src/self_update.rs:5-6`
  - `REPO = "quantum-box/tachyon-apps"`
  - `TAG_PREFIX = "tachyon-cli-v"` (hardcoded、prefix 違いは即 error)
- このコードは `https://api.github.com/repos/quantum-box/tachyon-apps/releases/latest`
  を fetch するが、`tachyon-apps` repo の latest release は repo-wide な
  `v0.71.1` (release-please monorepo 全体タグ) を返す。
- 結果、`v0.71.1`.strip_prefix("tachyon-cli-v") = None → `bail!`。

新 SSoT である `tachyon-sdk/cli/src/install_cli.rs` 側にも 2 つの問題:

1. `tag.strip_prefix('v').unwrap_or(tag)` は `tachyon-cli-v0.5.0` を strip
   できず、version 比較が常に false → 毎回ダウンロードに行く (実害なし
   だが意図とズレ)。
2. CLI の subcommand 名が `install` のみで、ユーザーが叩く `self-update` が
   存在しない (clap が `error: unrecognized subcommand`)。

加えて `tachyon-cli-v0.5.0` の release は手動 publish された "Mirror"。
auto-release-cli.yml は `v$VERSION` tag しか作らないため、CLI 単独タグ
prefix での自動公開ルートが未整備 (= Done(2) のために workflow 修正必須)。

## Fix scope (tachyon-sdk repo, freeze 済 tachyon-apps は触らない)

1. `cli/src/install_cli.rs`: prefix parser を `tachyon-cli-v` (primary) →
   `v` (fallback) の順に strip。
2. `cli/src/main.rs`: `Install` コマンドを `SelfUpdate` にリネームし
   `#[command(name = "self-update", visible_alias = "install")]` で両対応。
3. `.github/workflows/auto-release-cli.yml`: tag を `tachyon-cli-v$VERSION` に。
4. `.github/workflows/release-cli.yml`: trigger を `tachyon-cli-v*` に。
5. `cli/Cargo.toml`: version `0.2.2` → `0.5.1` (latest publish より上、
   self-update verify で実 upgrade を観測するため)。

## Done definition (PdM 指示 3 条件)

1. tag prefix 整合 fix (PR merged、CI green)
2. tag publish 1 件 (`tachyon-cli-v0.5.1` を auto-release で push)
3. `tachyon self-update` 実バイナリ実行 → Success stdout/stderr 添付

## 進捗

- [x] Root cause primary-source 特定
- [ ] taskdoc commit (このファイル)
- [ ] install_cli.rs + main.rs パッチ
- [ ] release workflows パッチ
- [ ] Cargo.toml bump
- [ ] PR open + CI green + admin merge
- [ ] tachyon-cli-v0.5.1 release publish 確認
- [ ] self-update 実機 verify (stdout/stderr 添付)
- [ ] PR description + 本 taskdoc に verify ログ貼付
- [ ] Linear PLT-923 Done flip + self-kill
