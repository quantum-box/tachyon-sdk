import type { AgentEvent, TachyonAgentClientConfig } from './types'

export class TachyonAgentClient {
	private readonly apiKey: string
	private readonly baseUrl: string

	constructor({ apiKey, baseUrl }: TachyonAgentClientConfig) {
		this.apiKey = apiKey
		this.baseUrl = baseUrl.replace(/\/+$/, '')
	}

	async *streamAgent(prompt: string): AsyncIterable<AgentEvent> {
		const response = await fetch(`${this.baseUrl}/v1/agent/execute`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				Authorization: `Bearer ${this.apiKey}`,
				Accept: 'text/event-stream',
			},
			body: JSON.stringify({ prompt }),
		})

		if (!response.ok) {
			const body = await response.text().catch(() => '')
			throw new Error(
				`streamAgent failed: ${response.status} ${response.statusText}${body ? ` - ${body}` : ''}`,
			)
		}

		yield* readSSEStream(response)
	}
}

async function* readSSEStream(response: Response): AsyncIterable<AgentEvent> {
	const body = response.body
	if (!body) throw new Error('Response body is null')

	const reader = body.getReader()
	const decoder = new TextDecoder()
	let buffer = ''

	try {
		while (true) {
			const { done, value } = await reader.read()
			if (done) break

			buffer += decoder.decode(value, { stream: true })

			const lines = buffer.split('\n')
			buffer = lines.pop() ?? ''

			for (const line of lines) {
				const trimmed = line.trim()
				if (!trimmed || trimmed.startsWith(':')) continue
				if (trimmed.startsWith('data:')) {
					const data = trimmed.slice(5).trim()
					if (data === '[DONE]') return
					try {
						yield JSON.parse(data) as AgentEvent
					} catch {
						// skip malformed JSON
					}
				}
			}
		}

		if (buffer.trim().startsWith('data:')) {
			const data = buffer.trim().slice(5).trim()
			if (data !== '[DONE]') {
				try {
					yield JSON.parse(data) as AgentEvent
				} catch {
					// skip malformed JSON
				}
			}
		}
	} finally {
		reader.releaseLock()
	}
}
