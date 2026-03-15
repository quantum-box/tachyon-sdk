import type { AgentChatClient } from './AgentChatClient'

export function createChatroomApi(client: AgentChatClient) {
	return {
		createChatRoom: (title?: string) => client.createChatRoom(title),
		getChatrooms: () => client.getChatrooms(),
		updateChatRoom: (id: string, payload: { name?: string }) =>
			client.updateChatRoom(id, payload),
		deleteChatroom: (id: string) => client.deleteChatroom(id),
	}
}
