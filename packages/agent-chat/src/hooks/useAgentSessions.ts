import { useCallback, useState } from 'react'
import type { AgentSession } from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'

export function useAgentSessions() {
	const { client } = useAgentChatContext()
	const [sessions, setSessions] = useState<AgentSession[]>([])
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<Error | null>(null)

	const fetchSessions = useCallback(
		async (limit = 20, offset = 0) => {
			setIsLoading(true)
			setError(null)
			try {
				const page = await client.getSessions(limit, offset)
				setSessions(current =>
					offset === 0
						? page
						: [
								...current.filter(
									session => !page.some(next => next.id === session.id),
								),
								...page,
							],
				)
				return page
			} catch (cause) {
				const failure =
					cause instanceof Error ? cause : new Error(String(cause))
				setError(failure)
				throw failure
			} finally {
				setIsLoading(false)
			}
		},
		[client],
	)

	const createSession = useCallback(
		async (name?: string) => {
			setError(null)
			try {
				const result = await client.createSession(name)
				setSessions(current => [
					{
						id: result.session_id,
						name: result.name,
						created_at: result.created_at,
						updated_at: result.created_at,
					},
					...current,
				])
				return result
			} catch (cause) {
				const failure =
					cause instanceof Error ? cause : new Error(String(cause))
				setError(failure)
				throw failure
			}
		},
		[client],
	)

	const deleteSession = useCallback(
		async (id: string) => {
			setError(null)
			try {
				await client.deleteSession(id)
				setSessions(current => current.filter(session => session.id !== id))
			} catch (cause) {
				const failure =
					cause instanceof Error ? cause : new Error(String(cause))
				setError(failure)
				throw failure
			}
		},
		[client],
	)

	return {
		sessions,
		isLoading,
		error,
		fetchSessions,
		createSession,
		deleteSession,
	}
}
