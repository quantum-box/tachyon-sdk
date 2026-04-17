---
name: image-gen
description: "AI画像生成スキル。tachyon image generate CLIでAI画像を生成し、ローカル保存またはTachyon Storageにアップロード。「画像生成」「ロゴ案」「ビジュアル素材」「Hero画像」「image-gen」等でトリガー。"
---

# Image Gen Skill

`tachyon image generate` CLI を使ってAI画像を生成するスキル。

## 前提

- CLI: `tachyon` (`~/.local/bin/tachyon` または `$PATH` に存在)
- 認証: `tachyon login` 済、または `TACHYON_API_KEY` 設定済
- テナント: `TACHYON_TENANT_ID` 設定済

## コマンドリファレンス

```bash
tachyon image generate \
  --prompt "説明テキスト" \
  --model gpt-image-1.5 \
  --size 1024x1024 \
  --quality high \
  --n 1 \
  --output /path/to/output.png \
  [--storage]
```

### オプション

| フラグ | デフォルト | 説明 |
|--------|-----------|------|
| `--prompt` / `-p` | (必須) | 生成する画像の説明テキスト |
| `--model` / `-m` | `gpt-image-1.5` | 使用モデル |
| `--size` | — | 画像サイズ: `1024x1024`, `1024x1536`, `1536x1024`, `auto` |
| `--quality` | — | 品質: `low`, `medium`, `high` |
| `--n` | `1` | 生成枚数 (1–10) |
| `--output` / `-o` | — | ローカル保存パス (複数枚は `_1`, `_2` サフィックス) |
| `--storage` | `false` | Tachyon Storage にアップロード |

### 対応モデル

| モデル | 特徴 |
|--------|------|
| `gpt-image-1.5` | 高品質・デフォルト |
| `gpt-image-1` | バランス型 |
| `gpt-image-1-mini` | 高速・軽量 |
| `grok-2-image` | Grok 画像モデル (出力: webp) |
| `gemini-2.0-flash-exp-image-generation` | Gemini 実験モデル |

## 利用パターン

### パターン1: ローカルファイルに保存

```bash
tachyon image generate \
  --prompt "modern SaaS landing page hero, dark theme, glassmorphism" \
  --model gpt-image-1.5 \
  --quality high \
  --output ~/Downloads/hero.png
```

### パターン2: Tachyon Storage にアップロード

```bash
tachyon image generate \
  --prompt "minimalist logo for a cloud platform, blue gradient" \
  --model gpt-image-1.5 \
  --size 1024x1024 \
  --storage
# → Storage key と公開URL を返す
```

### パターン3: 複数枚生成

```bash
tachyon image generate \
  --prompt "product mockup for mobile app" \
  --n 4 \
  --output ~/assets/mockup.png
# → mockup_1.png ~ mockup_4.png が生成される
```

## 出力フォーマット

```
Generating image with model: gpt-image-1.5
Prompt: ...

Generated 1 image(s) using gpt-image-1.5
Cost: $0.040000

--- Image 1 ---
URL: https://...
  Saved to: /path/to/output.png
```

## エラー対処

| エラー | 対処 |
|--------|------|
| `401 Unauthorized` | `tachyon login` で再認証 |
| `403 Forbidden` | テナントIDを確認: `TACHYON_TENANT_ID` |
| `400 Bad Request` | `--size` の値が対応モデルで使えるか確認 |
| `Storage upload failed` | ネットワーク接続・Storage権限を確認 |
