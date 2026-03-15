import type { AgentChatClient } from './AgentChatClient'
import type { RegenerateInsightsRequest } from './types'

export function createInsightApi(client: AgentChatClient) {
	return {
		getInsights: () => client.getInsights(),
		regenerateInsights: (payload?: RegenerateInsightsRequest) =>
			client.regenerateInsights(payload),
	}
}
