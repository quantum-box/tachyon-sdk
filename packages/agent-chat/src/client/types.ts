/** Token usage information */
export type Usage = {
	prompt_tokens: number
	completion_tokens: number
	total_tokens: number
}

/** Sub-agent source metadata */
export type AgentSource = {
	chatroom_id: string
}

/** A chunk of agent output (streaming or stored) */
export type AgentChunk = {
	type:
		| 'user'
		| 'ask'
		| 'thinking'
		| 'tool_call'
		| 'tool_call_args'
		| 'tool_result'
		| 'tool_call_pending'
		| 'say'
		| 'attempt_completion'
		| 'assistant'
		| 'completion'
		| 'usage'
		| 'tool_job_started'
	id: string
	text?: string
	created_at: string
	user_id?: string
	thinking?: string
	tool_name?: string
	tool_arguments?: string
	tool_result?: string
	tool_id?: string
	args?: unknown
	content?: string
	result?: string
	command?: string | null
	index?: number
	is_finished?: boolean
	isStreaming?: boolean
	options?: string[]
	// usage fields
	prompt_tokens?: number
	completion_tokens?: number
	total_tokens?: number
	total_cost?: number
	// tool_job_started fields
	job_id?: string
	provider?: string
	// Sub-agent metadata
	agent?: AgentSource
}

/** Response from creating a chat room */
export type CreateChatRoomResponse = {
	chatroom: {
		id: string
		name: string
	}
}

/** Request payload for executing an agent task */
export type AgentExecuteRequest = {
	task: string
	agent_protocol_id?: string
	agent_protocol_mode?: 'disabled' | 'auto' | 'manual'
	user_custom_instructions?: string
	assistant_name?: string
	model?: string
	mcp_hub_config_json?: string
	additional_tool_description?: string
	auto_approve?: boolean
	max_requests?: number
	tool_access?: ToolAccess
}

/** Tool access permissions for agent execution */
export type ToolAccess = {
	filesystem?: boolean
	command?: boolean
	coding_agent_job?: boolean
	agent_protocol?: boolean
	web_search?: boolean
	url_fetch?: boolean
	sub_agent?: boolean
}

/** Information about a single LLM model */
export type ModelInfo = {
	id: string
	name: string
	provider: string
	supported_features: string[]
	description?: string | null
	context_window?: number | null
	max_output_tokens?: number | null
}

/** A chat room */
export type ChatRoom = {
	id: string
	name: string
	created_at: string
}

/** Response from listing chat rooms */
export type GetChatroomsResponse = {
	chatrooms: ChatRoom[]
}

/** Response from listing agent messages */
export type AgentMessagesResponse = {
	messages: AgentChunk[]
}

/** Agent execution status */
export type AgentStatusResponse = {
	is_running: boolean
	progress?: number
	state?: string
}

/** Response from deleting messages */
export type DeleteMessagesResponse = {
	deleted_count: number
	deleted_message_ids: string[]
}

/** Response from restoring messages */
export type RestoreMessagesResponse = {
	restored_count: number
	restored_message_ids: string[]
}

/** Response from retrying an agent message */
export type RetryAgentMessageResponse = {
	deleted_count: number
	deleted_message_ids: string[]
	retry_prompt: string | null
}

/** Request payload for retrying an agent message */
export type RetryAgentMessageRequest = {
	new_prompt?: string
	model?: string
}

/** A chat message (non-agent format) */
export type ChatMessage = {
	id: string
	chatroom_id: string
	role: 'system' | 'user' | 'assistant'
	content: unknown
	user_id: string
	created_at: string
}

/** Options for filtering models */
export type GetModelsOptions = {
	/** Supported features to filter by. Defaults to `['agent']` when omitted. */
	supportedFeatures?: string[]
	/** Only return models with a catalog product. Defaults to `true` when omitted. */
	requireCatalogProduct?: boolean
}

// ── Saved Memory ────────────────────────────────────────────────────

/** A saved memory clause */
export type SavedMemory = {
	id: string
	clause: string
	raw_facts: string[]
	source: 'MANUAL' | 'CHAT'
	status: 'ACTIVE' | 'ARCHIVED'
	created_at: string
	updated_at: string
}

/** Request payload for creating a saved memory */
export type CreateMemoryRequest = {
	clause: string
	raw_facts?: string[]
	source?: 'MANUAL' | 'CHAT'
}

/** Response from listing memories */
export type GetMemoriesResponse = {
	memories: SavedMemory[]
}

// ── Agent Protocols ─────────────────────────────────────────────────

/** An agent protocol definition */
export type AgentProtocol = {
	id: string
	title: string
	protocol_name: string
	description?: string | null
	markdown: string
	created_at: string
	updated_at: string
}

/** Request payload for creating an agent protocol */
export type CreateAgentProtocolRequest = {
	title: string
	protocol_name: string
	description?: string
	markdown: string
}

/** Request payload for updating an agent protocol */
export type UpdateAgentProtocolRequest = {
	title?: string
	protocol_name?: string
	description?: string
	markdown?: string
}

/** Response from listing agent protocols */
export type GetAgentProtocolsResponse = {
	items: AgentProtocol[]
	next_cursor?: string | null
}

/** Options for listing agent protocols */
export type GetAgentProtocolsOptions = {
	keyword?: string
	cursor?: string
	limit?: number
}

// ── User Insights ───────────────────────────────────────────────────

/** A user insight generated from conversation history */
export type UserInsight = {
	id: string
	summary: string
	confidence: string
	started_at?: string | null
	ended_at?: string | null
	message_count: number
	cluster_keywords: string[]
	last_generated_at: string
	created_at: string
	updated_at: string
}

/** Response from listing/regenerating insights */
export type GetInsightsResponse = {
	insights: UserInsight[]
}

/** Request payload for regenerating insights */
export type RegenerateInsightsRequest = {
	model?: string
}

// ── Tool Result ─────────────────────────────────────────────────────

/** Request payload for submitting a tool result */
export type SubmitToolResultRequest = {
	tool_id: string
	result: string
	is_error?: boolean
}

/** Response from submitting a tool result */
export type SubmitToolResultResponse = {
	status: 'accepted'
}

// ── Tool Jobs ───────────────────────────────────────────────────────

/** Tool job status */
export type ToolJobStatus =
	| 'queued'
	| 'running'
	| 'succeeded'
	| 'failed'
	| 'cancelled'

/** Tool job billing information */
export type ToolJobBilling = {
	estimated_nanodollar?: number | null
	observed_nanodollar?: number | null
}

/** A tool job (coding agent job) */
export type ToolJob = {
	id: string
	provider: string
	prompt: string
	status: ToolJobStatus
	context_paths: string[]
	output_profile?: string | null
	environment: Record<string, string>
	normalized_output?: { format: string; body: unknown } | null
	raw_events?: unknown[] | null
	artifacts?: unknown[] | null
	billing?: ToolJobBilling | null
	error_message?: string | null
	session_id?: string | null
	resume_session_id?: string | null
	use_worktree: boolean
	auto_merge: boolean
	assigned_worker_id?: string | null
	created_at: string
	updated_at: string
}

/** Request payload for creating a tool job */
export type CreateToolJobRequest = {
	provider: string
	prompt: string
	context_paths?: string[]
	output_profile?: string
	environment?: Record<string, string>
	metadata?: unknown
	resume_session_id?: string
	use_worktree?: boolean
	auto_merge?: boolean
	worker_id?: string
}

/** Response from listing tool jobs */
export type GetToolJobsResponse = {
	jobs: ToolJob[]
}

/** Response from creating a tool job */
export type CreateToolJobResponse = {
	job: ToolJob
}

/** Response from getting a single tool job */
export type GetToolJobResponse = {
	job: ToolJob
}

// ── Tool Search ─────────────────────────────────────────────────────

/** A single tool in a search result group */
export type ToolSearchTool = {
	name: string
	description: string
	input_schema?: unknown
}

/** A grouped tool search result by server */
export type ToolSearchResultGroup = {
	server_name: string
	tools: ToolSearchTool[]
	total_matches: number
}

/** Request payload for searching tools */
export type SearchToolsRequest = {
	query?: string
	server_name?: string
	limit?: number
	mcp_config_json: string
}

/** Response from searching tools */
export type SearchToolsResponse = {
	results: ToolSearchResultGroup[]
}
