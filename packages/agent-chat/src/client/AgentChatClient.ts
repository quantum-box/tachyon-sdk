import type {
	AgentChunk,
	AgentExecuteRequest,
	AgentMessagesResponse,
	AgentProtocol,
	AgentStatusResponse,
	ChatMessage,
	ChatRoom,
	CreateAgentProtocolRequest,
	CreateChatRoomResponse,
	CreateMemoryRequest,
	CreateToolJobRequest,
	CreateToolJobResponse,
	DeleteMessagesResponse,
	GetAgentProtocolsOptions,
	GetAgentProtocolsResponse,
	GetChatroomsResponse,
	GetInsightsResponse,
	GetMemoriesResponse,
	GetModelsOptions,
	GetToolJobResponse,
	GetToolJobsResponse,
	ModelInfo,
	RegenerateInsightsRequest,
	RestoreMessagesResponse,
	RetryAgentMessageRequest,
	RetryAgentMessageResponse,
	SavedMemory,
	SearchToolsRequest,
	SearchToolsResponse,
	SubmitToolResultRequest,
	SubmitToolResultResponse,
	ToolJob,
	UpdateAgentProtocolRequest,
	UserInsight,
} from './types'

export type AgentChatClientConfig = {
	/** Base URL of the API server, e.g. "http://localhost:50054" */
	apiBaseUrl: string
	/** Bearer token for authentication */
	accessToken: string
	/** Operator tenant ID (tn_xxx) */
	tenantId: string
	/** Optional user ID header */
	userId?: string
}

export class AgentChatClient {
	private config: AgentChatClientConfig

	constructor(config: AgentChatClientConfig) {
		this.config = config
	}

	// ── Internal helpers ──────────────────────────────────────────────

	private getHeaders(): Record<string, string> {
		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${this.config.accessToken}`,
			'x-operator-id': this.config.tenantId,
		}
		if (this.config.userId) {
			headers['x-user-id'] = this.config.userId
		}
		return headers
	}

	private buildUrl(path: string): string {
		const base = this.config.apiBaseUrl.replace(/\/+$/, '')
		return `${base}${path}`
	}

	private async request<T>(path: string, options?: RequestInit): Promise<T> {
		const url = this.buildUrl(path)
		const response = await fetch(url, {
			...options,
			headers: {
				...this.getHeaders(),
				...options?.headers,
			},
		})
		if (!response.ok) {
			const body = await response.text().catch(() => '')
			throw new Error(
				`AgentChatClient request failed: ${response.status} ${response.statusText}${body ? ` - ${body}` : ''}`,
			)
		}
		return (await response.json()) as T
	}

	// ── Chatroom API ──────────────────────────────────────────────────

	async createChatRoom(title?: string): Promise<CreateChatRoomResponse> {
		return this.request<CreateChatRoomResponse>('/v1/llms/chatrooms', {
			method: 'POST',
			body: JSON.stringify({
				...(title && { name: title }),
			}),
		})
	}

	async updateChatRoom(
		chatRoomId: string,
		payload: { name?: string },
	): Promise<ChatRoom> {
		const data = await this.request<{ chatroom: ChatRoom }>(
			`/v1/llms/chatrooms/${chatRoomId}`,
			{
				method: 'PATCH',
				body: JSON.stringify(payload),
			},
		)
		return data.chatroom
	}

	async getChatrooms(): Promise<ChatRoom[]> {
		const data = await this.request<GetChatroomsResponse>('/v1/llms/chatrooms')
		return data.chatrooms
	}

	async deleteChatroom(chatRoomId: string): Promise<void> {
		const url = this.buildUrl(`/v1/llms/chatrooms/${chatRoomId}`)
		const response = await fetch(url, {
			method: 'DELETE',
			headers: this.getHeaders(),
		})
		if (!response.ok) {
			const body = await response.text().catch(() => '')
			throw new Error(
				`Failed to delete chatroom: ${response.status}${body ? ` - ${body}` : ''}`,
			)
		}
	}

	// ── Agent API ─────────────────────────────────────────────────────

	async executeAgent(
		chatRoomId: string,
		args: AgentExecuteRequest,
	): Promise<Response> {
		const url = this.buildUrl(`/v1/llms/sessions/${chatRoomId}/agent/execute`)
		const response = await fetch(url, {
			method: 'POST',
			headers: {
				...this.getHeaders(),
				Accept: 'text/event-stream',
			},
			body: JSON.stringify(args),
		})
		if (!response.ok) {
			const body = await response.text().catch(() => '')
			throw new Error(
				`Agent execute failed: ${response.status}${body ? ` - ${body}` : ''}`,
			)
		}
		return response
	}

	async getMessages(chatRoomId: string): Promise<AgentChunk[]> {
		const data = await this.request<AgentMessagesResponse>(
			`/v1/llms/sessions/${chatRoomId}/agent/messages`,
		)
		return data.messages
	}

	async getStatus(chatRoomId: string): Promise<AgentStatusResponse> {
		return this.request<AgentStatusResponse>(
			`/v1/llms/sessions/${chatRoomId}/agent/status`,
		)
	}

	async getModels(options?: GetModelsOptions): Promise<ModelInfo[]> {
		const query = buildModelsQuery(options)
		const path = `/v1/llms/models${query ? `?${query}` : ''}`
		const data = await this.request<{
			models: ModelInfo[]
			total_count: number
		}>(path)
		return data.models
	}

	async deleteMessagesAfter(
		chatRoomId: string,
		messageId: string,
	): Promise<DeleteMessagesResponse> {
		return this.request<DeleteMessagesResponse>(
			`/v1/llms/chatrooms/${chatRoomId}/messages?after=${messageId}`,
			{ method: 'DELETE' },
		)
	}

	async restoreMessages(
		chatRoomId: string,
		messageId: string,
	): Promise<RestoreMessagesResponse> {
		return this.request<RestoreMessagesResponse>(
			`/v1/llms/chatrooms/${chatRoomId}/messages/${messageId}/restore`,
			{ method: 'POST' },
		)
	}

	async retryAgentMessage(
		chatRoomId: string,
		messageId: string,
		request?: RetryAgentMessageRequest,
	): Promise<RetryAgentMessageResponse> {
		return this.request<RetryAgentMessageResponse>(
			`/v1/llms/chatrooms/${chatRoomId}/messages/${messageId}/retry`,
			{
				method: 'POST',
				body: JSON.stringify(request ?? {}),
			},
		)
	}

	async getDeletedMessages(chatRoomId: string): Promise<AgentChunk[]> {
		const data = await this.request<{
			messages: AgentChunk[]
		}>(`/v1/llms/chatrooms/${chatRoomId}/messages/deleted`)
		return data.messages
	}

	// ── Saved Memory API ─────────────────────────────────────────────

	async getMemories(status?: 'ACTIVE' | 'ARCHIVED'): Promise<SavedMemory[]> {
		const query = status ? `?status=${status}` : ''
		const data = await this.request<GetMemoriesResponse>(
			`/v1/agent/memory${query}`,
		)
		return data.memories
	}

	async createMemory(payload: CreateMemoryRequest): Promise<SavedMemory> {
		return this.request<SavedMemory>('/v1/agent/memory', {
			method: 'POST',
			body: JSON.stringify(payload),
		})
	}

	async deleteMemory(memoryId: string): Promise<void> {
		const url = this.buildUrl(`/v1/agent/memory/${memoryId}`)
		const response = await fetch(url, {
			method: 'DELETE',
			headers: this.getHeaders(),
		})
		if (!response.ok) {
			const body = await response.text().catch(() => '')
			throw new Error(
				`Failed to delete memory: ${response.status}${body ? ` - ${body}` : ''}`,
			)
		}
	}

	async archiveMemory(memoryId: string): Promise<SavedMemory> {
		return this.request<SavedMemory>(`/v1/agent/memory/${memoryId}/archive`, {
			method: 'POST',
		})
	}

	async activateMemory(memoryId: string): Promise<SavedMemory> {
		return this.request<SavedMemory>(`/v1/agent/memory/${memoryId}/activate`, {
			method: 'POST',
		})
	}

	// ── Agent Protocols API ──────────────────────────────────────────

	async getAgentProtocols(
		options?: GetAgentProtocolsOptions,
	): Promise<GetAgentProtocolsResponse> {
		const params = new URLSearchParams()
		if (options?.keyword) params.set('keyword', options.keyword)
		if (options?.cursor) params.set('cursor', options.cursor)
		if (options?.limit) params.set('limit', String(options.limit))
		const query = params.toString()
		return this.request<GetAgentProtocolsResponse>(
			`/v1/llms/agent-protocols${query ? `?${query}` : ''}`,
		)
	}

	async getAgentProtocol(protocolId: string): Promise<AgentProtocol> {
		return this.request<AgentProtocol>(`/v1/llms/agent-protocols/${protocolId}`)
	}

	async createAgentProtocol(
		payload: CreateAgentProtocolRequest,
	): Promise<AgentProtocol> {
		return this.request<AgentProtocol>('/v1/llms/agent-protocols', {
			method: 'POST',
			body: JSON.stringify(payload),
		})
	}

	async updateAgentProtocol(
		protocolId: string,
		payload: UpdateAgentProtocolRequest,
	): Promise<AgentProtocol> {
		return this.request<AgentProtocol>(
			`/v1/llms/agent-protocols/${protocolId}`,
			{
				method: 'PATCH',
				body: JSON.stringify(payload),
			},
		)
	}

	async deleteAgentProtocol(protocolId: string): Promise<void> {
		const url = this.buildUrl(`/v1/llms/agent-protocols/${protocolId}`)
		const response = await fetch(url, {
			method: 'DELETE',
			headers: this.getHeaders(),
		})
		if (!response.ok) {
			const body = await response.text().catch(() => '')
			throw new Error(
				`Failed to delete agent protocol: ${response.status}${body ? ` - ${body}` : ''}`,
			)
		}
	}

	// ── User Insights API ────────────────────────────────────────────

	async getInsights(): Promise<UserInsight[]> {
		const data = await this.request<GetInsightsResponse>('/v1/agent/insights')
		return data.insights
	}

	async regenerateInsights(
		payload?: RegenerateInsightsRequest,
	): Promise<UserInsight[]> {
		const data = await this.request<GetInsightsResponse>(
			'/v1/agent/insights/regenerate',
			{
				method: 'POST',
				body: JSON.stringify(payload ?? {}),
			},
		)
		return data.insights
	}

	// ── Tool Result API ──────────────────────────────────────────────

	async submitToolResult(
		chatRoomId: string,
		payload: SubmitToolResultRequest,
	): Promise<SubmitToolResultResponse> {
		return this.request<SubmitToolResultResponse>(
			`/v1/llms/chatrooms/${chatRoomId}/agent/tool-result`,
			{
				method: 'POST',
				body: JSON.stringify(payload),
			},
		)
	}

	// ── Tool Jobs API ────────────────────────────────────────────────

	async createToolJob(payload: CreateToolJobRequest): Promise<ToolJob> {
		const data = await this.request<CreateToolJobResponse>(
			'/v1/agent/tool-jobs',
			{
				method: 'POST',
				body: JSON.stringify(payload),
			},
		)
		return data.job
	}

	async getToolJobs(): Promise<ToolJob[]> {
		const data = await this.request<GetToolJobsResponse>('/v1/agent/tool-jobs')
		return data.jobs
	}

	async getToolJob(jobId: string): Promise<ToolJob> {
		const data = await this.request<GetToolJobResponse>(
			`/v1/agent/tool-jobs/${jobId}`,
		)
		return data.job
	}

	async cancelToolJob(jobId: string): Promise<ToolJob> {
		return this.request<ToolJob>(`/v1/agent/tool-jobs/${jobId}/cancel`, {
			method: 'POST',
		})
	}

	async *streamToolJob(jobId: string): AsyncGenerator<AgentChunk> {
		const url = this.buildUrl(
			`/v1/agent/tool-jobs/${jobId}/stream?format=agent_chunk`,
		)
		const response = await fetch(url, {
			headers: {
				...this.getHeaders(),
				Accept: 'text/event-stream',
			},
		})
		if (!response.ok) {
			const body = await response.text().catch(() => '')
			throw new Error(
				`Tool job stream failed: ${response.status}${body ? ` - ${body}` : ''}`,
			)
		}
		yield* this.readSSEStream(response)
	}

	// ── Message Update API ───────────────────────────────────────────

	async updateMessage(
		chatRoomId: string,
		messageId: string,
		payload: { content: string },
	): Promise<ChatMessage> {
		return this.request<ChatMessage>(
			`/v1/llms/chatrooms/${chatRoomId}/messages/${messageId}`,
			{
				method: 'PUT',
				body: JSON.stringify(payload),
			},
		)
	}

	// ── Tool Search API ──────────────────────────────────────────────

	async searchTools(payload: SearchToolsRequest): Promise<SearchToolsResponse> {
		return this.request<SearchToolsResponse>('/v1/agent/tools/search', {
			method: 'POST',
			body: JSON.stringify(payload),
		})
	}

	// ── SSE Streaming ─────────────────────────────────────────────────

	async *streamAgent(
		chatRoomId: string,
		args: AgentExecuteRequest,
	): AsyncGenerator<AgentChunk> {
		const response = await this.executeAgent(chatRoomId, args)
		yield* this.readSSEStream(response)
	}

	// ── Private SSE reader ────────────────────────────────────────────

	private async *readSSEStream(response: Response): AsyncGenerator<AgentChunk> {
		const body = response.body
		if (!body) {
			throw new Error('Response body is null')
		}

		const reader = body.getReader()
		const decoder = new TextDecoder()
		let buffer = ''

		try {
			while (true) {
				const { done, value } = await reader.read()
				if (done) break

				buffer += decoder.decode(value, { stream: true })

				const lines = buffer.split('\n')
				// Keep the last incomplete line in the buffer
				buffer = lines.pop() ?? ''

				for (const line of lines) {
					const trimmed = line.trim()
					if (!trimmed || trimmed.startsWith(':')) {
						// Empty line or SSE comment, skip
						continue
					}
					if (trimmed.startsWith('data:')) {
						const data = trimmed.slice(5).trim()
						if (data === '[DONE]') {
							return
						}
						try {
							const chunk = JSON.parse(data) as AgentChunk
							yield chunk
						} catch {
							// Skip malformed JSON lines
						}
					}
				}
			}

			// Process any remaining data in the buffer
			if (buffer.trim()) {
				const trimmed = buffer.trim()
				if (trimmed.startsWith('data:')) {
					const data = trimmed.slice(5).trim()
					if (data !== '[DONE]') {
						try {
							const chunk = JSON.parse(data) as AgentChunk
							yield chunk
						} catch {
							// Skip malformed JSON
						}
					}
				}
			}
		} finally {
			reader.releaseLock()
		}
	}
}

// ── Private helpers ─────────────────────────────────────────────────

function buildModelsQuery(options?: GetModelsOptions): string {
	const params = new URLSearchParams()

	const hasSupportedFeaturesOption =
		options !== undefined && options !== null && 'supportedFeatures' in options
	const initialFeatures = hasSupportedFeaturesOption
		? (options?.supportedFeatures ?? [])
		: ['agent']

	let normalizedFeatures = initialFeatures
		.map(f => f?.trim())
		.filter((f): f is string => Boolean(f))

	const hasRequireCatalogProductOption =
		options !== undefined &&
		options !== null &&
		'requireCatalogProduct' in options
	const requireCatalogProduct = hasRequireCatalogProductOption
		? (options?.requireCatalogProduct ?? false)
		: true

	if (
		requireCatalogProduct &&
		!normalizedFeatures.some(f => f.toLowerCase() === 'agent')
	) {
		normalizedFeatures = [...normalizedFeatures, 'agent']
	}

	for (const feature of normalizedFeatures) {
		params.append('supported_feature', feature)
	}

	if (requireCatalogProduct) {
		params.append('require_agent_product', 'true')
	}

	return params.toString()
}
