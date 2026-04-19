#!/usr/bin/env node
/**
 * @quantumbox/tachyon-agent — Node.js CLI example
 *
 * Usage:
 *   export TACHYON_API_KEY=your-api-key
 *   node index.js "Slackに通知して"
 *
 * Or pipe:
 *   echo "最新のデプロイ状況を教えて" | node index.js
 */
import { streamAgent } from '@quantumbox/tachyon-agent'

const apiKey = process.env.TACHYON_API_KEY
const baseUrl = process.env.TACHYON_API_URL ?? 'https://api.n1.tachy.one'

if (!apiKey) {
  console.error('Error: TACHYON_API_KEY is not set')
  process.exit(1)
}

// Accept prompt from CLI arg or stdin
const prompt = process.argv[2] ?? (await readStdin())

if (!prompt) {
  console.error('Usage: node index.js "<prompt>"')
  process.exit(1)
}

console.error(`[tachyon-agent] prompt: ${prompt}\n`)

let toolCallCount = 0

for await (const event of streamAgent(prompt, { apiKey, baseUrl })) {
  switch (event.type) {
    case 'text':
      process.stdout.write(event.content)
      break
    case 'tool_call':
      toolCallCount++
      process.stderr.write(`[tool_call #${toolCallCount}] ${event.content}\n`)
      break
    case 'done':
      process.stdout.write('\n')
      break
  }
}

async function readStdin() {
  const chunks = []
  for await (const chunk of process.stdin) {
    chunks.push(chunk)
  }
  return Buffer.concat(chunks).toString('utf8').trim()
}
