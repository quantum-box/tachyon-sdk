# PLT-929: install.sh GitHub API 60 req/hr/IP rate-limit 回避

Linear: https://linear.app/quantum-box/issue/PLT-929
Priority: Urgent / Status: In Progress

## 症状

CEO による onboarding 動作確認時:

```
$ curl -fsSL https://github.com/quantum-box/tachyon-sdk/raw/main/scripts/install.sh | sh
...
HTTP/1.1 403 rate limit exceeded
Failed to fetch latest release tag from https://api.github.com/repos/quantum-box/tachyon-sdk/releases/latest.
```

`install.sh` が `https://api.github.com/repos/${REPO}/releases/latest` を curl で叩いて
`tag_name` を抽出していたため、anonymous IP 60 req/hr 制限に当たり 403 で blocked。
新規 user の onboarding flow を即時詰まらせる CEO blocker。

## Root cause (primary-source verified)

`scripts/install.sh` (修正前):

- L38: `API_URL="https://api.github.com/repos/${REPO}/releases/latest"`
- L44-50: `curl -fsSL ... "$API_URL" | grep '"tag_name"' | sed ...` で tag 抽出
- API 経由 = anonymous で 60 req/hr/IP の hard limit (GitHub doc 既知)
- `GITHUB_TOKEN` env で fallback 可だが新規 user UX 最悪 (token 取得手順を強制)

## Fix 方針 (CEO 承認 path 2)

**path (2): `/releases/latest/download/<asset>` URL を直 fetch (API 不使用)**

GitHub は `https://github.com/${REPO}/releases/latest/download/<asset>` を
HTML 302 で latest release asset (`releases/download/<tag>/<asset>`) に redirect する。
API call 不要 → rate limit 完全回避 (release-assets.githubusercontent.com への
S3 署名付き URL に着地、anonymous でも到達可)。

primary-source verify (本日 2026-04-26 09:51 JST):

```
$ curl -sSLI https://github.com/quantum-box/tachyon-sdk/releases/latest/download/tachyon-linux-x86_64.tar.gz
HTTP/2 302
location: https://github.com/quantum-box/tachyon-sdk/releases/download/tachyon-cli-v0.5.1/tachyon-linux-x86_64.tar.gz
...
HTTP/2 200
```

`gh api repos/quantum-box/tachyon-sdk/releases/latest` で
`tag_name=tachyon-cli-v0.5.1` / `prerelease=false` / `draft=false` を確認済 →
redirect は安定。

## 修正前後の install.sh 差分要約

修正前 (L34-57, 24 行):

- `API_URL=https://api.github.com/...` ← 削除
- `AUTH_HEADER` / `GITHUB_TOKEN` block ← 削除 (path 2 では不要)
- `LATEST_TAG="$(curl ... | grep tag_name | sed ...)"` ← 削除
- `LATEST_TAG` 空チェック / GITHUB_TOKEN retry hint error message ← 削除
- `DOWNLOAD_URL=".../releases/download/${LATEST_TAG}/..."` ←
  `.../releases/latest/download/...` に置換

修正後:

- `DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARTIFACT}.tar.gz"`
  1 行のみ。
- 解決後 tag は `curl -w '%{url_effective}'` で post-download に取得し log 表示。

`grep -F 'api.github.com' scripts/install.sh` → 0 件 (Done 1 verify)。

## Done 定義

1. install.sh の API call 削除完了 (`grep api.github.com` で 0 件)
2. `/releases/latest/download/<asset>` URL 経由で binary download 成功
   (linux-x86_64 / linux-arm64 / darwin-arm64 各 1 回、size + sha256 verify)
3. 60 連続 install で 403 0 件 (`grep -c 403` = 0、`grep -c "Installed"` = 60)

## 進捗

- [x] Root cause primary-source 特定
- [x] taskdoc 初稿 commit (このファイル)
- [ ] install.sh patch
- [ ] linux-x86_64 実 install verify (`tachyon --version`)
- [ ] linux-arm64 / darwin-arm64 download + extract + sha256 verify
      (host = linux-x86_64 のため arch 違いの execute は不可、
      download/extract/file-type 確認のみ)
- [ ] 60 連続 install 実機 verify (403 count = 0)
- [ ] PR open + CI green + admin merge
- [ ] Linear PLT-929 Done flip + self-kill

## 実機 verify ログ

(patch 後追記)

## PR / commit / Done flip

(後追記)
