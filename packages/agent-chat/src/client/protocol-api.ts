import type { AgentChatClient } from './AgentChatClient'
import type {
	CreateAgentProtocolRequest,
	GetAgentProtocolsOptions,
	UpdateAgentProtocolRequest,
} from './types'

export function createProtocolApi(client: AgentChatClient) {
	return {
		getAgentProtocols: (options?: GetAgentProtocolsOptions) =>
			client.getAgentProtocols(options),
		getAgentProtocol: (id: string) => client.getAgentProtocol(id),
		createAgentProtocol: (payload: CreateAgentProtocolRequest) =>
			client.createAgentProtocol(payload),
		updateAgentProtocol: (id: string, payload: UpdateAgentProtocolRequest) =>
			client.updateAgentProtocol(id, payload),
		deleteAgentProtocol: (id: string) => client.deleteAgentProtocol(id),
	}
}
