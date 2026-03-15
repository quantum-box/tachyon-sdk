import { useCallback, useState } from 'react'
import type { CreateToolJobRequest, ToolJob } from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'

export function useToolJobs() {
	const { client } = useAgentChatContext()
	const [jobs, setJobs] = useState<ToolJob[]>([])
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<Error | null>(null)

	const fetchToolJobs = useCallback(async () => {
		setIsLoading(true)
		setError(null)
		try {
			const result = await client.getToolJobs()
			setJobs(result)
			return result
		} catch (e) {
			const err = e instanceof Error ? e : new Error(String(e))
			setError(err)
			throw err
		} finally {
			setIsLoading(false)
		}
	}, [client])

	const createToolJob = useCallback(
		async (payload: CreateToolJobRequest) => {
			setError(null)
			try {
				const job = await client.createToolJob(payload)
				setJobs(prev => [...prev, job])
				return job
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	const cancelToolJob = useCallback(
		async (id: string) => {
			setError(null)
			try {
				const updated = await client.cancelToolJob(id)
				setJobs(prev => prev.map(j => (j.id === id ? updated : j)))
				return updated
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	return {
		jobs,
		isLoading,
		error,
		fetchToolJobs,
		createToolJob,
		cancelToolJob,
	}
}
