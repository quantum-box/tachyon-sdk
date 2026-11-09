import type { AgentChatClient } from './AgentChatClient'

export function createSessionApi(client: AgentChatClient) {
	return {
		createSession: (name?: string) => client.createSession(name),
		getSessions: (limit?: number, offset?: number) =>
			client.getSessions(limit, offset),
		getSession: (id: string) => client.getSession(id),
		deleteSession: (id: string) => client.deleteSession(id),
	}
}
