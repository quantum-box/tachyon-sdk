---
title: "PLT-1778 CLI manifest validate/plan/apply/reconcile"
type: "feature"
emoji: "📜"
topics: ["tachyon-cli", "manifest", "iac", "auth"]
published: true
targetFiles:
  - "cli/src/main.rs"
  - "cli/src/manifest/"
  - "cli/src/reconcile_cli.rs"
  - "cli/src/commands/auth/manifest.rs"
  - "cli/src/compute_cli.rs"
github: "https://github.com/quantum-box/tachyon-apps/issues/4935"
---

# PLT-1778: CLI manifest apply/reconcile 汎用導線

Linear: [PLT-1778](https://linear.app/quantum-box/issue/PLT-1778)  
GitHub: [tachyon-apps#4935](https://github.com/quantum-box/tachyon-apps/issues/4935)

## 背景

manifest 適用の主導線が `tachyon auth manifest apply`（auth 専用に見える）、`tachyon compute apps apply`（Cloud Apps 専用）、`tachyon reconcile`（Cloud Apps + auth の薄いラッパー）に分裂している。backend の `ApplyManifest` usecase は複数 kind を一括処理できるが、CLI UX が追いついていない。

## 目標

ユーザーが「manifest を apply したい」ときの **主導線** を `tachyon manifest` 配下に統一する。既存コマンドは互換維持（alias / deprecation 誘導のみ）。

```bash
tachyon manifest validate [--file PATH]...
tachyon manifest plan    [--file PATH] [--prune] [--dry-run] [--json]
tachyon manifest apply   [--file PATH] [--prune] [--dry-run] [--json]
tachyon manifest reconcile [--file PATH] [--prune] [--dry-run] [--json]
```

---

## Task 1 仕様（本 doc）

### 1. コマンド責務

| サブコマンド | 責務 | API 呼び出し |
|-------------|------|-------------|
| `validate` | ローカル schema / 構文検証のみ。tenant 不要可 | なし |
| `plan` | desired（repo）vs live（API）の差分表示。変更なし | 読み取りのみ |
| `apply` | desired を API に反映（create/update） | 書き込み |
| `reconcile` | apply と同じ discovery。drift 修復・定期運用向け。`--dry-run` 時は plan 相当 | 書き込み（dry-run 時は読み取り） |

**plan vs reconcile vs apply の違い（docs 必須）**

- `plan`: 人間が差分をレビューする read-only。CI で「変更があること」だけ検知してもよい。
- `apply`: 明示的な一度きりの適用。ローカル manifest を SSoT として push。
- `reconcile`: 運用・定期実行向けの語彙。現行 `tachyon reconcile` と同じ「複数ソースを順に適用」だが discovery を統一。将来的に ops reconcile（CF KV 等）とは **名前空間を分離**（`tachyon ops reconcile` 等は別 issue）。

### 2. Manifest discovery（統一ルール）

**探索起点**: カレントディレクトリから親へ `.git` まで遡り `tachyon.yml` を探す（既存 `find_tachyon_yml` と同じ）。

**`--file` 未指定（auto-discovery）**

| 順序 | ソース | 判定 | ハンドラ（Phase 2 実装） |
|------|--------|------|-------------------------|
| 1 | `tachyon.yml` | `kind: CloudApps \| CloudApp` | `compute_cli::AppsCommand::Apply/Plan` 相当 |
| 2 | `tachyon.yml` | `auth.manifest` セクション | `auth::manifest` merge/validate |
| 3 | `.tachyon/manifests/**/*.{yml,yaml}` | auth flat schema または k8s ActionSet/Policy | `auth::manifest` |
| 4 | `.tachyon/manifests/**/*.{yml,yaml}` | `apiVersion: apps.tachy.one/v1alpha` | Phase 2b: `iac_cli` / `ApplyManifest` API 経由 |
| 5 | その他 kind | 未対応 | `validate` で warn、`apply` で skip + 明示メッセージ |

**`--file PATH` 指定時**

- **単一ファイルモード**: 指定 path のみ読み込み。親ディレクトリの他 manifest は **対象外**（現行 `reconcile --file` が Cloud Apps のみで auth は auto-discovery する挙動は **廃止**）。
- 複数 `--file` は Phase 2 では非対応（将来拡張）。1 ファイルのみ。
- 相対 path は cwd 基準。

**マルチアプリ CloudApps**

- 既存 `--app` フラグを `manifest apply/plan/reconcile` に継承。
- `--environment` も Cloud Apps ハンドラに渡す（default: `sandbox`）。

### 3. 実行順序（apply / reconcile）

複数 manifest が discovery された場合、**決定論的な固定順**で実行する（`HashMap` 順禁止）:

1. IaC `v1alpha` manifests（`.tachyon/manifests` 内、ファイル名昇順）
2. Auth manifest（merged single desired state）
3. Cloud Apps / CloudApp（`tachyon.yml`）

各ステップは前ステップが fatal error でなければ続行。`--json` 時はステップごとに結果オブジェクトを配列で返す。

### 4. フラグ共通仕様

| フラグ | validate | plan | apply | reconcile |
|--------|----------|------|-------|-----------|
| `--file` | o | o | o | o |
| `--prune` | - | o | o | o（auth prune は現状 unsupported → warn） |
| `--dry-run` | - | o（plan と同義） | o | o |
| `--json` | o | o | o | o |
| `--app` | - | o | o | o |
| `--environment` | - | o | o | o |

**Tenant 解決**: `plan` / `apply` / `reconcile` は active tenant 必須（現行 `auth manifest` / `reconcile` と同様）。`validate` は tenant 不要。

### 5. 既存コマンドとの互換

| 既存 | 扱い |
|------|------|
| `tachyon auth manifest {fmt,validate,plan,apply}` | **維持**。help に「prefer `tachyon manifest`」 |
| `tachyon compute apps apply` | **維持**。内部は変更不要（Phase 2 で manifest から delegate 可） |
| `tachyon reconcile` | **alias** → `tachyon manifest reconcile` に delegate（挙動変更は discovery 統一のみ） |
| `tachyon iac apply/plan` | Phase 2 では触らない。Phase 3 で `manifest apply` への統合を検討 |

### 6. Backend マッピング（Phase 2b 以降）

| Manifest kind | 現行 CLI 経路 | 目標 |
|---------------|--------------|------|
| Auth actions/policies | auth REST via `auth/manifest.rs` | `manifest apply` から既存関数を呼ぶ |
| CloudApp(s) | `compute_cli` apps apply | 同上 |
| IaC v1alpha (Operator, Policy, ApiKey, …) | `iac_cli apply` / server `ApplyManifest` | 段階的に `manifest apply` から呼ぶ |
| `spec.auth` on CloudApp | server-side in `apply_manifest.rs` | Cloud Apps apply に含まれる（CLI 変更不要） |

Phase 2 最小スコープ: **auth + CloudApps の orchestration のみ**。IaC v1alpha は discovery 登録 + validate のみ、apply は「未対応」メッセージ。

### 7. モジュール構成（Phase 2 実装案）

```
cli/src/manifest/
  mod.rs           # ManifestArgs, subcommands
  discovery.rs     # 統合 discovery（本仕様 §2）
  validate.rs
  plan.rs
  apply.rs
  reconcile.rs     # apply と共有 + reconcile_cli から移行
```

`main.rs` に `Manifest(manifest::ManifestArgs)` トップレベルコマンドを追加。

### 8. テスト計画

- Unit: discovery 優先順・`--file` 単一ファイル・kind 判定（fixture YAML）
- Integration: 既存 `auth manifest` / `compute apps apply` / `reconcile` の golden 出力が変わらないこと
- Integration: 新 `manifest apply` が auth + CloudApps 順で同等結果

### 9. 受け入れ条件（issue より）

- [ ] 「manifest apply」が auth 専用に見えない主導線がある
- [ ] `reconcile` と `manifest reconcile` の関係が help/docs で明確
- [ ] auth / CloudApp / IaC の責務が docs で一貫
- [ ] 既存 `auth manifest apply` / `compute apps apply` が破壊されない

---

## 実装フェーズ

| Phase | 内容 | 状態 |
|-------|------|------|
| **1** | 本 taskdoc（仕様固定） | **In Progress** |
| 2a | `cli/src/manifest/` + validate/plan/apply/reconcile（auth + CloudApps） | Pending |
| 2b | IaC v1alpha discovery + validate | Pending |
| 3 | docs + `reconcile` alias + 互換テスト | Pending |
| 4 | IaC apply を `ApplyManifest` API に統合 | Optional |

## 進捗

- [x] Task 1 仕様 taskdoc 化
- [ ] Phase 2a CLI 実装
- [ ] PR → CI → merge
- [ ] tachyon-apps docs 更新（auth-manifest-cli.md 等）
