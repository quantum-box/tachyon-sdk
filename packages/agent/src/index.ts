import { TachyonAgentClient } from './TachyonAgentClient'
import type { AgentEvent, TachyonAgentClientConfig } from './types'

export { TachyonAgentClient }
export type { AgentEvent, AgentEventType, TachyonAgentClientConfig } from './types'

export function streamAgent(
	prompt: string,
	config: TachyonAgentClientConfig,
): AsyncIterable<AgentEvent> {
	return new TachyonAgentClient(config).streamAgent(prompt)
}
