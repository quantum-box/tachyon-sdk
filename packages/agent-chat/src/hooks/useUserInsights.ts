import { useCallback, useState } from 'react'
import type { RegenerateInsightsRequest, UserInsight } from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'

export function useUserInsights() {
	const { client } = useAgentChatContext()
	const [insights, setInsights] = useState<UserInsight[]>([])
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<Error | null>(null)

	const fetchInsights = useCallback(async () => {
		setIsLoading(true)
		setError(null)
		try {
			const result = await client.getInsights()
			setInsights(result)
			return result
		} catch (e) {
			const err = e instanceof Error ? e : new Error(String(e))
			setError(err)
			throw err
		} finally {
			setIsLoading(false)
		}
	}, [client])

	const regenerateInsights = useCallback(
		async (payload?: RegenerateInsightsRequest) => {
			setIsLoading(true)
			setError(null)
			try {
				const result = await client.regenerateInsights(payload)
				setInsights(result)
				return result
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			} finally {
				setIsLoading(false)
			}
		},
		[client],
	)

	return {
		insights,
		isLoading,
		error,
		fetchInsights,
		regenerateInsights,
	}
}
