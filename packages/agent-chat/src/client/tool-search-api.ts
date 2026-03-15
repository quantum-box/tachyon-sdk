import type { AgentChatClient } from './AgentChatClient'
import type { SearchToolsRequest } from './types'

export function createToolSearchApi(client: AgentChatClient) {
	return {
		searchTools: (payload: SearchToolsRequest) => client.searchTools(payload),
	}
}
