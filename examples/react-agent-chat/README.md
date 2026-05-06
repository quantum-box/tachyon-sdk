# react-agent-chat example

React + Vite example for `@tachyon-sdk/agent` with SSE streaming.

## What You Learn

- How to call `streamAgent()` from a browser UI
- How to append streamed text to an assistant message
- How to keep user input disabled while streaming
- How to start from a minimal React example before adding auth, history, and tenant logic

## Setup

```bash
git clone https://github.com/quantum-box/tachyon-sdk.git
cd tachyon-sdk/examples/react-agent-chat
npm install

cp .env.example .env
```

Edit `.env`:

```env
VITE_TACHYON_API_KEY=your-api-key
VITE_TACHYON_API_URL=https://api.n1.tachy.one
```

## Run

```bash
npm run dev
```

Open `http://localhost:5173`.

## Main Code Path

The example calls `streamAgent()` and appends `event.content` to the last assistant message.

```typescript
for await (const event of streamAgent(prompt, {
  apiKey: API_KEY,
  baseUrl: BASE_URL,
})) {
  if (event.type === "text") {
    setMessages((prev) => {
      const next = [...prev];
      const last = next[next.length - 1];
      if (last?.role === "assistant") {
        next[next.length - 1] = {
          ...last,
          content: last.content + event.content,
        };
      }
      return next;
    });
  }

  if (event.type === "done") break;
}
```

## Production Note

This example keeps the API key in a Vite environment variable to make local setup short. In production, do not expose secret keys in the browser. Put a backend proxy between your frontend and Tachyon Agent API, then enforce your own auth, tenant, rate limit, and audit policy there.

## Related

- Node CLI example: `../node-cli`
- Package: `@tachyon-sdk/agent`
