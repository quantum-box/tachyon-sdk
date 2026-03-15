import { type ReactNode, createContext, useContext, useMemo } from 'react'
import {
	AgentChatClient,
	type AgentChatClientConfig,
} from '../client/AgentChatClient'
import type { AgentChunk, ToolAccess } from '../client/types'

export type AgentChatConfig = AgentChatClientConfig & {
	defaultModel?: string
	context?: {
		page: string
		hint?: string
		data?: Record<string, unknown>
	}
	toolAccess?: ToolAccess
	onError?: (error: Error) => void
	onMessage?: (message: AgentChunk) => void
}

type AgentChatContextValue = {
	client: AgentChatClient
	config: AgentChatConfig
}

const AgentChatContext = createContext<AgentChatContextValue | null>(null)

export function AgentChatProvider({
	children,
	...config
}: AgentChatConfig & { children: ReactNode }) {
	const client = useMemo(
		() =>
			new AgentChatClient({
				apiBaseUrl: config.apiBaseUrl,
				accessToken: config.accessToken,
				tenantId: config.tenantId,
				userId: config.userId,
			}),
		[config.apiBaseUrl, config.accessToken, config.tenantId, config.userId],
	)

	const value = useMemo(() => ({ client, config }), [client, config])

	return (
		<AgentChatContext.Provider value={value}>
			{children}
		</AgentChatContext.Provider>
	)
}

export function useAgentChatContext() {
	const ctx = useContext(AgentChatContext)
	if (!ctx) {
		throw new Error('useAgentChatContext must be used within AgentChatProvider')
	}
	return ctx
}
