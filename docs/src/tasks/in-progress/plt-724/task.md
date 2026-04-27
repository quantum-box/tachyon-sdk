# PLT-724: tachyon CLI multi-account (profile) support — Phase 1

Linear: https://linear.app/quantum-box/issue/PLT-724
Priority: High / Status: In Progress

## 背景

2026-04-21 MOVERENT デプロイ停滞調査時、PdM-PF2 が `invalid_grant` で詰まった
事例。tachyon CLI が single account 前提で、別アカウント切替には毎回
`tachyon login` で credentials.json を上書きするしかなく、複数 tenant /
複数 user を行き来する CEO / PdM / leader workflow に対応できていない。

gcloud / aws / kubectl に倣い、profile-based auth context を導入する。

## SSoT

- repo = `quantum-box/tachyon-sdk` (worktree `~/tachyon-sdk.leader-plt724`)
- branch = `feature/plt-724` (origin/main から分岐済、a355b61)
- 主戦場 = `cli/src/auth.rs` 拡張 + `cli/src/main.rs` の subcommand 追加
- `tachyon-apps/apps/tachyon-cli/` は freeze (subtree split 残骸、絶対参照禁止)

## 現状調査 (primary-source)

`cli/src/auth.rs` (a355b61):

- `credentials_path()` → `dirs::config_dir()/tachyon/credentials.json`
  (linux: `~/.config/tachyon/credentials.json`)
- `StoredCredentials` { access_token, refresh_token, id_token, expires_at,
  token_type, operator_id }
- `load_credentials()` / `save_credentials()` / `login()` / `logout()` /
  `refresh_access_token()` がいずれも 1 つの credentials.json 直決め打ち
- `0o600` permissions on save (unix)

`cli/src/main.rs`:

- 1 個の `tachyon login` / `tachyon logout` flat subcommand
- top-level に `--profile` flag は無し、`TACHYON_PROFILE` env も無し
- `resolve_token()` で `auth::load_credentials()` 呼出し → profile 概念無し

callers:
- `cli/src/main.rs:103` `auth::load_credentials()` (resolve_token)
- `cli/src/switch_cli.rs:65,68` `load_credentials/save_credentials`
- `cli/src/resolve.rs:77` `load_credentials` (tenant resolution fallback)

## 設計 (Phase 1)

### Storage layout

```
~/.config/tachyon/
├── credentials.json          # legacy (auto-migrated to profiles/default.json on first read, then removed)
├── active_profile            # plain text: profile name (e.g. "work\n"). 無ければ "default"
└── profiles/
    ├── default.json          # StoredCredentials shape
    ├── work.json
    └── personal.json
```

注: 仕様書で `~/.tachyon/profiles/<name>.json` と書かれているが、既存実装が
`dirs::config_dir()/tachyon/` (linux: `~/.config/tachyon`) を使っているため、
backward compat 優先で `dirs::config_dir()/tachyon/profiles/` を採用する。
これは macOS では `~/Library/Application Support/tachyon/profiles/` になり、
gcloud (`~/.config/gcloud`) と同じ XDG 系の規約に揃う。

### Profile resolution priority

`tachyon` プロセス起動時の active profile 決定順:

1. `--profile <name>` global flag (clap で `env = "TACHYON_PROFILE"` を併設)
2. `TACHYON_PROFILE` env var (clap が 1 で吸収)
3. `~/.config/tachyon/active_profile` file
4. `"default"`

### 自動マイグレーション (backward compat)

profile 操作のたびに最初に `migrate_legacy_credentials()` を呼ぶ:

- `credentials.json` が存在し、`profiles/default.json` が無ければ
  `credentials.json` の中身を読んで `profiles/default.json` に書き出し、
  legacy `credentials.json` は残す (rollback 可能性のため。後続 PR で削除可)。
- `active_profile` が無ければ `"default"` を書き込む。

これで既存 single-account user は `tachyon` を一度走らせるだけで
default profile としてシームレスに移行する。

### Public API (auth.rs 拡張)

```rust
pub fn profiles_dir() -> Result<PathBuf>;
pub fn profile_path(name: &str) -> Result<PathBuf>;
pub fn active_profile_path() -> Result<PathBuf>;

pub fn read_active_profile() -> Result<String>;   // file 無 or 空 → "default"
pub fn write_active_profile(name: &str) -> Result<()>;

pub fn list_profiles() -> Result<Vec<String>>;     // sorted
pub fn load_profile(name: &str) -> Result<Option<StoredCredentials>>;
pub fn save_profile(name: &str, creds: &StoredCredentials) -> Result<()>;
pub fn delete_profile(name: &str) -> Result<bool>; // true if removed

pub fn migrate_legacy_if_needed() -> Result<()>;
pub fn validate_profile_name(name: &str) -> Result<()>; // [a-zA-Z0-9_.-]{1,64}
```

既存の `load_credentials() / save_credentials()` は **active profile に対する
shim** として残す (callers の影響を最小化):

```rust
pub fn load_credentials() -> Result<Option<StoredCredentials>> {
    migrate_legacy_if_needed()?;
    load_profile(&read_active_profile()?)
}

pub fn save_credentials(creds: &StoredCredentials) -> Result<()> {
    migrate_legacy_if_needed()?;
    save_profile(&read_active_profile()?, creds)
}
```

`login(oauth_config, api_url, profile)` は引数に profile name を取り、終わりに
`save_profile(profile, creds)` する。`logout(profile)` は `delete_profile`。

### CLI subcommand

```
tachyon auth login [--profile <name>]
tachyon auth logout [--profile <name>]
tachyon auth list
tachyon auth use <profile>

# backward compat: 既存の top-level も残す
tachyon login [--profile <name>]
tachyon logout [--profile <name>]

# 1-shot override
tachyon --profile <name> compute apps list
TACHYON_PROFILE=work tachyon compute apps list
```

実装:

- `Cli` struct に `#[arg(long, env = "TACHYON_PROFILE")] profile: Option<String>` を追加
- `Commands::Auth(AuthArgs)` を追加、`Login` / `Logout` は thin alias で残す
- `resolve_token()` は `cli.profile` を見て、None の場合のみ `read_active_profile()`
  に fall back

### Token refresh と profile

`refresh_access_token` は同じ profile に上書き保存する必要があるので、
シグネチャを `refresh_access_token(oauth_config, creds, profile_name)` に変更。

## 実装計画

1. `auth.rs` に profile API 追加 + 既存 fn の shim 化 (with migration)
2. `main.rs` に `Auth` subcommand + `--profile` global flag 追加
3. `resolve.rs` / `switch_cli.rs` の load/save 呼出は shim 経由なので原則無変更
   (profile は active_profile fileに反映済の前提)
4. unit test: profile_path / active_profile / migration / list_profiles /
   validate_profile_name (XDG_CONFIG_HOME を tempdir に差し替え)
5. integration test (cli/tests/profiles.rs): `auth list` empty → after migration
   shows "default" → `auth use foo` で active 切替 (profile 無くてもファイル
   作成のみは許可、login 前は load_profile が None を返す動作で OK)
6. README に profile section 追加
7. CHANGELOG / knowledge docs (`~/knowledge/tachyon-cli-multi-profile.md`) 追加

## Phase 2 (本 PR scope 外)

Linear 子 issue として分離 (本 PR description にも明記):

- macOS Keychain integration (`security` framework)
- Linux secret-service (`libsecret` / `keyring` crate)
- credential 暗号化保管
- `auth status` コマンド (token 残存時間表示)

`auth.rs` 内に `// TODO(PLT-724-phase2):` コメント残しのみ。

## Done 定義 (Phase 1)

1. `cargo build` 通過 (no warnings on touched files)
2. `cargo clippy --all-targets -- -D warnings` 通過
3. `cargo test` (unit + integration) 通過
4. CI 全 job green
5. PR merge
6. 本番 (cargo install or release binary) E2E 7 項目 PASS:
   1. `tachyon auth login --profile work` + `tachyon auth login --profile personal`
      で 2 profile 並行登録
   2. `tachyon auth list` で 2 profile 表示
   3. `tachyon auth use personal` → `tachyon auth list` で active 印切替
   4. `TACHYON_PROFILE=work tachyon auth list` で env override 動作
   5. `tachyon --profile personal auth list` で 1-shot flag 動作
   6. `tachyon auth logout --profile personal` → list で 1 profile 残
   7. backward compat: 既存 `~/.config/tachyon/credentials.json` mock 配置 →
      `tachyon auth list` で `default` profile 自動移行表示

## 進捗

- 2026-04-27 着手、worktree 作成、現状調査済、本 doc commit 予定
