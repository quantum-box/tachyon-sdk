import type { AgentChatClient } from './AgentChatClient'
import type { CreateMemoryRequest } from './types'

export function createMemoryApi(client: AgentChatClient) {
	return {
		getMemories: (status?: 'ACTIVE' | 'ARCHIVED') => client.getMemories(status),
		createMemory: (payload: CreateMemoryRequest) =>
			client.createMemory(payload),
		deleteMemory: (id: string) => client.deleteMemory(id),
		archiveMemory: (id: string) => client.archiveMemory(id),
		activateMemory: (id: string) => client.activateMemory(id),
	}
}
