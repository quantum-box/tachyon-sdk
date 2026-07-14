import { useCallback, useState } from 'react'
import type { CreateCodingJobRequest, CodingJob } from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'

export function useCodingJobs() {
	const { client } = useAgentChatContext()
	const [jobs, setJobs] = useState<CodingJob[]>([])
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<Error | null>(null)

	const fetchCodingJobs = useCallback(async () => {
		setIsLoading(true)
		setError(null)
		try {
			const result = await client.getCodingJobs()
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

	const createCodingJob = useCallback(
		async (payload: CreateCodingJobRequest) => {
			setError(null)
			try {
				const job = await client.createCodingJob(payload)
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

	const cancelCodingJob = useCallback(
		async (id: string) => {
			setError(null)
			try {
				const updated = await client.cancelCodingJob(id)
				setJobs(prev => prev.map(j => (j.coding_job_id === id ? updated : j)))
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
		fetchCodingJobs,
		createCodingJob,
		cancelCodingJob,
	}
}
