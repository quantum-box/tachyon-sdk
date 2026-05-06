# node-cli example

Node.js streaming CLI for `@tachyon-sdk/agent`.

## What You Learn

- How to call `streamAgent()` from Node.js
- How to stream agent text to `stdout`
- How to keep tool/debug events on `stderr`
- How to turn a CLI pattern into a Slack bot, LINE bot, CI job, or internal automation backend

## Setup

```bash
git clone https://github.com/quantum-box/tachyon-sdk.git
cd tachyon-sdk/examples/node-cli
npm install

export TACHYON_API_KEY=your-api-key
export TACHYON_API_URL=https://api.n1.tachy.one
```

## Usage

Pass a prompt as an argument:

```bash
node index.js "Slackに本番デプロイ完了を通知して"
```

Or pipe from stdin:

```bash
echo "最新のデプロイ状況は？" | node index.js
```

## Output Contract

- `stdout`: agent text response
- `stderr`: prompt, tool call events, and debug information

This lets you pipe the agent response into another command without mixing debug output into the result.

## Main Code Path

```javascript
import { streamAgent } from "@tachyon-sdk/agent";

for await (const event of streamAgent(prompt, { apiKey, baseUrl })) {
  switch (event.type) {
    case "text":
      process.stdout.write(event.content);
      break;
    case "tool_call":
      process.stderr.write(`[tool_call] ${event.content}\n`);
      break;
    case "done":
      process.stdout.write("\n");
      break;
  }
}
```

## Common Adaptations

- Replace CLI args with a Slack `app.message()` handler
- Replace stdin with a LINE webhook payload
- Save the final response into a CI artifact
- Wrap `streamAgent()` with your own retry / timeout / audit logging

## Related

- React example: `../react-agent-chat`
- Package: `@tachyon-sdk/agent`
