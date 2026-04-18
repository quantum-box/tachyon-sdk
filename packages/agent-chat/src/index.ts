// API Client
export { AgentChatClient } from './client/AgentChatClient'
export type { AgentChatClientConfig } from './client/AgentChatClient'

// API factory functions
export { createChatroomApi } from './client/chatroom-api'
export { createAgentApi } from './client/agent-api'
export { createMemoryApi } from './client/memory-api'
export { createProtocolApi } from './client/protocol-api'
export { createInsightApi } from './client/insight-api'
export { createToolJobApi } from './client/tool-job-api'
export { createToolSearchApi } from './client/tool-search-api'

// Types
export type {
	AgentChunk,
	AgentSource,
	ChatRoom,
	ChatMessage,
	AgentExecuteRequest,
	AgentStatusResponse,
	ToolAccess,
	GetModelsOptions,
	ModelInfo,
	CreateChatRoomResponse,
	GetChatroomsResponse,
	AgentMessagesResponse,
	DeleteMessagesResponse,
	RestoreMessagesResponse,
	RetryAgentMessageResponse,
	RetryAgentMessageRequest,
	// Saved Memory
	SavedMemory,
	CreateMemoryRequest,
	GetMemoriesResponse,
	// Agent Protocols
	AgentProtocol,
	CreateAgentProtocolRequest,
	UpdateAgentProtocolRequest,
	GetAgentProtocolsResponse,
	GetAgentProtocolsOptions,
	// User Insights
	UserInsight,
	GetInsightsResponse,
	RegenerateInsightsRequest,
	// Tool Result
	SubmitToolResultRequest,
	SubmitToolResultResponse,
	// Tool Jobs
	ToolJobStatus,
	ToolJobBilling,
	ToolJob,
	CreateToolJobRequest,
	CreateToolJobResponse,
	GetToolJobsResponse,
	GetToolJobResponse,
	// Tool Search
	ToolSearchTool,
	ToolSearchResultGroup,
	SearchToolsRequest,
	SearchToolsResponse,
} from './client/types'

// --- Providers ---
export {
	AgentChatProvider,
	useAgentChatContext,
} from './providers/AgentChatProvider'
export type { AgentChatConfig } from './providers/AgentChatProvider'

// --- Hooks ---
export { useAgentChat } from './hooks/useAgentChat'
export { useAgentStream } from './hooks/useAgentStream'
export type { UseAgentStreamOptions } from './hooks/useAgentStream'
export { useChatRoom } from './hooks/useChatRoom'
export { useSavedMemory } from './hooks/useSavedMemory'
export { useAgentProtocols } from './hooks/useAgentProtocols'
export { useUserInsights } from './hooks/useUserInsights'
export { useToolJobs } from './hooks/useToolJobs'
export {
	usePersisted,
	PERSISTENCE_KEYS,
} from './hooks/useChatPersistence'

// --- Components ---
export { AgentChat } from './components/AgentChat'
export type { AgentChatProps } from './components/AgentChat'
export { FloatingChatPanel } from './components/FloatingChatPanel'
export type {
	FloatingChatPanelProps,
	FloatingChatPanelPosition,
	AgentChatTheme,
} from './components/FloatingChatPanel'
export { ChatPanel } from './components/ChatPanel'
export { MessageList } from './components/MessageList'
export { MessageBubble } from './components/MessageBubble'
export { ChatInput } from './components/ChatInput'
export { ModelSelector } from './components/ModelSelector'
export { ThinkingIndicator } from './components/ThinkingIndicator'
export { ToolCallDisplay } from './components/ToolCallDisplay'
export { ToolResultDisplay } from './components/ToolResultDisplay'
export { UsageSummary } from './components/UsageSummary'
