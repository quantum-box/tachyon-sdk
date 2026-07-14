import type { AgentChatClient } from './AgentChatClient'
import type { CreateCodingJobRequest } from './types'

export function createCodingJobApi(client: AgentChatClient) {
	return {
		createCodingJob: (payload: CreateCodingJobRequest) =>
			client.createCodingJob(payload),
		getCodingJobs: () => client.getCodingJobs(),
		getCodingJob: (id: string) => client.getCodingJob(id),
		cancelCodingJob: (id: string) => client.cancelCodingJob(id),
		streamCodingJob: (id: string) => client.streamCodingJob(id),
	}
}
