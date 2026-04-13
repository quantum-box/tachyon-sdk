---
name: tachyon-cli
description: "Tachyon Cloud Appのビルド・デプロイ状況確認+ログ取得。`tachyon` CLIでCloud App(Compute)のデバッグ・修正ワークフローを自動化。「cloud appのログ」「デプロイ失敗」「ビルド失敗」「tachyon status」「tachyon logs」等でトリガー。"
---

# Tachyon CLI Skill

Cloud App (tachyon compute) のビルド・デプロイ状況を確認し、失敗時のログを取得して修正サイクルを回すためのスキル。

## 前提

- CLI: `/home/ubuntu/.local/bin/tachyon` (Rust製、`tachyon-apps/crates/tachyon-cli`由来)
- API: `https://api.n1.tachy.one` (default)
- 認証: 事前に `tachyon login` 済。未認証なら `TACHYON_API_KEY` or `tachyon login` を先に。
- App context: app配下に `.tachyon.json` があれば `--app-id` 省略可

## 主要コマンド

| 用途 | コマンド |
|---|---|
| 認証 | `tachyon login` |
| 現在のstatus | `tachyon status [--app-id X]` |
| ビルドログ | `tachyon logs --build-id <BUILD_ID>` |
| ランタイムログ | `tachyon logs --deployment-id <DEPLOYMENT_ID>` |
| 環境変数操作 | `tachyon env list / set KEY=VAL / unset KEY` |
| ドメイン | `tachyon domains list / add` |
| デプロイ | `tachyon deploy` |

## デバッグワークフロー

### 1. ビルド/デプロイ失敗の調査
```bash
cd <app-dir>  # .tachyon.json のある場所
tachyon status                       # 最新 build/deployment ID取得
tachyon logs --build-id <ID>         # ビルド失敗時
tachyon logs --deployment-id <ID>    # 起動/ランタイム失敗時
```

### 2. app-id指定(別ディレクトリから)
```bash
tachyon status --app-id <APP_ID>
tachyon logs --app-id <APP_ID> --build-id <BUILD_ID>
```

### 3. 環境変数起因の調査
```bash
tachyon env list --app-id <APP_ID>  # 設定漏れ確認(VITE_*, API keys等)
tachyon env set KEY=value
tachyon deploy                       # 再deploy
```

### 4. CEO/CTO/PdMへの報告フォーマット
失敗調査の報告には以下を含める:
- App ID / Build ID / Deployment ID
- 失敗フェーズ (Build / Deploy / Runtime)
- ログから該当エラー行引用(推測NG、実ログ必須)
- 次アクション(env追加・コード修正・インフラ対応)

## よくあるfailure patterns

| 症状 | 確認先 |
|---|---|
| Build失敗 | `tachyon logs --build-id` → Dockerfile/dependency/typecheck |
| Deploy失敗 | `tachyon logs --deployment-id` → env不足・port binding・migration |
| Runtime 500 | `tachyon logs --deployment-id` → runtime stack trace |
| White screen(frontend) | `tachyon env list` → VITE_* ハードコード確認 (cowork/PR#45事例) |

## 注意

- **ログ引用は実ログから**: CI失敗 root cause 報告ルールと同じ。推測でerror箇所を書かない
- **本番影響操作**(env変更・deploy)はPdM/COO承認必須
- **App IDをLinear/PRに残す**: 後追い調査で必要
