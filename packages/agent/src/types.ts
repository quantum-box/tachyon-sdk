export type AgentEventType = 'text' | 'tool_call' | 'done'

export type AgentEvent = {
	type: AgentEventType
	content: string
}

export type TachyonAgentClientConfig = {
	/** API key for authentication */
	apiKey: string
	/** Base URL of the Tachyon Agent API, e.g. "https://api.tachyon.ai" */
	baseUrl: string
}
