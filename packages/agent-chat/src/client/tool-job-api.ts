import type { AgentChatClient } from './AgentChatClient'
import type { CreateToolJobRequest } from './types'

export function createToolJobApi(client: AgentChatClient) {
	return {
		createToolJob: (payload: CreateToolJobRequest) =>
			client.createToolJob(payload),
		getToolJobs: () => client.getToolJobs(),
		getToolJob: (id: string) => client.getToolJob(id),
		cancelToolJob: (id: string) => client.cancelToolJob(id),
		streamToolJob: (id: string) => client.streamToolJob(id),
	}
}
