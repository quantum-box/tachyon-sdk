# PLT-927: tachyon-cli darwin-x86_64 release path 復活

Linear: https://linear.app/quantum-box/issue/PLT-927
Priority: Medium / Status: In Progress

## 背景

PLT-923 の follow-up。`macos-13` runner の queue が 10h+ で詰まり、
`tachyon-darwin-x86_64.tar.gz` の release asset が defer されたままになっている。
結果として macOS Intel ユーザーの `tachyon self-update` release path が欠落している。

## 方針

GitHub-hosted runner の現行 label に合わせ、`x86_64-apple-darwin` build を
`macos-15-intel` で復活させる。ubuntu cross compile は macOS SDK / linker 周りの
追加依存が大きいため、今回は既存の macOS native build workflow に戻す。

参照:

- GitHub Docs: hosted runner reference (`macos-15-intel` は Intel macOS runner)
- GitHub Changelog: `macos-13` runner image closing down
  - `macos-13` / `macos-13-large` / `macos-13-xlarge` は退役対象
  - x86_64 (Intel) が必要な場合の移行先として `macos-15-intel` が案内されている

## 修正対象

- `.github/workflows/auto-release-cli.yml`
- `.github/workflows/release-cli.yml`

## Done definition

1. `release-cli.yml` が `tachyon-darwin-x86_64` を build matrix に含む。
2. `auto-release-cli.yml` が同じ matrix を持ち、main push release でも同 asset を作る。
3. `macos-13` 依存を復活させない。
4. PR を作成し、CI Auto Release の安定化方針を説明する。

## 進捗

- [x] Linear issue と既存 workflow を確認
- [x] `macos-13` defer コメントの残存を確認
- [x] `macos-15-intel` + `x86_64-apple-darwin` matrix を復活
- [x] workflow YAML 検証
- [x] PR 作成

## 検証メモ

- `actionlint -ignore 'label "macos-15-intel" is unknown' .github/workflows/auto-release-cli.yml .github/workflows/release-cli.yml`: pass
- ローカルの `actionlint` は `macos-15-intel` label をまだ内蔵リストに持っていないため ignore 指定。GitHub 公式 docs / changelog では同 label が標準 Intel macOS runner として案内されている。
- PR: https://github.com/quantum-box/tachyon-sdk/pull/87
