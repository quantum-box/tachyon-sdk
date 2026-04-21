# node-cli example

Node.js streaming CLI for `@tachyon-sdk/agent`.

## Setup

```bash
cd examples/node-cli
npm install
export TACHYON_API_KEY=your-api-key
```

## Usage

```bash
# Pass prompt as argument
node index.js "Slackに本番デプロイ完了を通知して"

# Pipe from stdin
echo "最新のデプロイ状況は？" | node index.js

# Custom API URL (default: https://api.n1.tachy.one)
TACHYON_API_URL=https://api.n1.tachy.one node index.js "Hello"
```

## Output

- `stdout`: agent text response (streaming)
- `stderr`: tool call events + debug info
