# react-agent-chat example

React + Vite example for `@tachyon-sdk/agent` with SSE streaming.

## Setup

```bash
cd examples/react-agent-chat
npm install

# Create .env
cp .env.example .env
# Edit .env and set VITE_TACHYON_API_KEY
```

## .env

```
VITE_TACHYON_API_KEY=your-api-key
VITE_TACHYON_API_URL=https://api.n1.tachy.one
```

## Run

```bash
npm run dev
# Open http://localhost:5173
```

## Features

- SSE ストリーミング（文字が順次表示される）
- ユーザー / アシスタントのバブル UI
- Enter キーで送信
- ストリーミング中のカーソル表示（▋）
