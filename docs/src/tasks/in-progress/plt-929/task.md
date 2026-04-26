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
- [x] install.sh patch (本 commit)
- [x] linux-x86_64 実 install verify (`tachyon 0.5.1`)
- [x] linux-arm64 / darwin-arm64 download + extract + sha256 verify
      (host = linux-x86_64 のため arch 違いの execute は不可、
      download/extract/file-type 確認のみ)
- [x] 60 連続 install 実機 verify (403 count = 0)
- [x] PR open + CI green + admin merge (PR #56, squash 9c780fb @ 2026-04-26 09:59:32 UTC)
- [x] Linear PLT-929 Done flip + self-kill

## 実機 verify ログ (2026-04-26 JST、host: linux x86_64 Sakura VPS)

### Done(1) install.sh 内 api.github.com 削除

```
$ grep -cE 'api\.github\.com' scripts/install.sh
0
$ sh -n scripts/install.sh && echo OK
OK
```

### Done(2) platform 別 binary download 実機 verify

CDN 着地 (release-assets.githubusercontent.com S3 署名 URL) 経由で 200 OK / sha256
一致を 3 platform で確認。host arch の都合で execute verify は linux-x86_64 のみ
(他 2 arch は download + tar -xzf + `file(1)` で binary type 確認 + chmod 755)。

#### linux-x86_64 (full install)

```
$ bash scripts/install.sh
Downloading tachyon (linux/x86_64) from latest release...
Installed tachyon to /home/ubuntu/.local/bin/tachyon
$ ls -la ~/.local/bin/tachyon
-rwxr-xr-x 1 ubuntu ubuntu 18173288 Apr 26 18:54 /home/ubuntu/.local/bin/tachyon
$ tachyon --version
tachyon 0.5.1
```

URL: https://github.com/quantum-box/tachyon-sdk/releases/latest/download/tachyon-linux-x86_64.tar.gz
- tarball size: 6,521,020 bytes
- tarball sha256: `01f3094fafda3ca4956578daccbb5f5a16c921cf1e251188f69de7fccaafd7cc`
- binary sha256: `82ff24847c1b3585280f886ba752ba8543bc8cfc610d77680c2420f091100c66`

#### linux-arm64 (download + extract verify)

URL: https://github.com/quantum-box/tachyon-sdk/releases/latest/download/tachyon-linux-arm64.tar.gz
- HTTP: 200, tarball size: 6,266,017 bytes
- tarball sha256: `aecdecdf981521f65e4751f1319b5b8fd1ffdcdc0bc699894e41770a3c1ceea3`
- binary sha256: `bf562c20f6255fe306120acc4bdd76578bb13d7ab37b65d4c83cecc02a2ff27c`
- `file(1)`: `ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV), dynamically linked, ...`
- chmod 755 OK

#### darwin-arm64 (download + extract verify)

URL: https://github.com/quantum-box/tachyon-sdk/releases/latest/download/tachyon-darwin-arm64.tar.gz
- HTTP: 200, tarball size: 5,943,111 bytes
- tarball sha256: `0b947b3e92bc2d869c10df123eb88e1e3af7b6fceeb4228f3cd8e2accda6fa12`
- binary sha256: `a97072cd2ea8e640e32b63b1b11943c745ffece7216c369a7d27a96ed5ddf1a7`
- `file(1)`: `Mach-O 64-bit arm64 executable, flags:<NOUNDEFS|DYLDLINK|TWOLEVEL|PIE|HAS_TLV_DESCRIPTORS>`
- chmod 755 OK

### Done(3) 60 連続 install 実機 verify (rate-limit 突破確認)

```
$ for i in $(seq 1 60); do echo "=== run $i ==="; bash scripts/install.sh; done > loop60.log 2>&1
elapsed=57s
$ grep -c '^Installed tachyon to' loop60.log
60
$ grep -c '403' loop60.log
0
$ grep -ciE 'rate.?limit|403' loop60.log
0
$ grep -c '^\[run .* FAILED' loop60.log
0
```

success rate = 60/60 (100%) / 403 count = 0 / rate-limit hits = 0。
平均 install 時間 ≒ 57s / 60 ≒ 0.95s/run。
GitHub anonymous の `api.github.com` 60 req/hr/IP は呼ばないため、
60 回連続でも一切 throttle されないことを実機 verify 済。

## PR / commit / Done flip

- PR: https://github.com/quantum-box/tachyon-sdk/pull/56
- merge commit (squash on main): `9c780fbe37db8b0c322096f510051358112f8d30`
- mergedAt: `2026-04-26T09:59:32Z` (UTC) / `2026-04-26 18:59:32 JST`
- CI: `Check CLI` SUCCESS (1m08s, run 24953860546)
- merge mode: `gh pr merge 56 --squash --admin --delete-branch`
- Linear PLT-929 status: Done (this commit)
- self-kill target: `tmux kill-window -t work:8`
