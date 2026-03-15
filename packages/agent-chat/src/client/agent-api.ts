import type { AgentChatClient } from './AgentChatClient'
import type {
	AgentExecuteRequest,
	GetModelsOptions,
	RetryAgentMessageRequest,
} from './types'

export function createAgentApi(client: AgentChatClient) {
	return {
		executeAgent: (chatRoomId: string, args: AgentExecuteRequest) =>
			client.executeAgent(chatRoomId, args),
		streamAgent: (chatRoomId: string, args: AgentExecuteRequest) =>
			client.streamAgent(chatRoomId, args),
		getMessages: (chatRoomId: string) => client.getMessages(chatRoomId),
		getStatus: (chatRoomId: string) => client.getStatus(chatRoomId),
		getModels: (options?: GetModelsOptions) => client.getModels(options),
		deleteMessagesAfter: (chatRoomId: string, messageId: string) =>
			client.deleteMessagesAfter(chatRoomId, messageId),
		restoreMessages: (chatRoomId: string, messageId: string) =>
			client.restoreMessages(chatRoomId, messageId),
		retryAgentMessage: (
			chatRoomId: string,
			messageId: string,
			req?: RetryAgentMessageRequest,
		) => client.retryAgentMessage(chatRoomId, messageId, req),
		getDeletedMessages: (chatRoomId: string) =>
			client.getDeletedMessages(chatRoomId),
	}
}
