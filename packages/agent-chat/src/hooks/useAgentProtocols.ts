import { useCallback, useState } from 'react'
import type {
	AgentProtocol,
	CreateAgentProtocolRequest,
	GetAgentProtocolsOptions,
	UpdateAgentProtocolRequest,
} from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'

export function useAgentProtocols() {
	const { client } = useAgentChatContext()
	const [protocols, setProtocols] = useState<AgentProtocol[]>([])
	const [nextCursor, setNextCursor] = useState<string | null | undefined>(null)
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<Error | null>(null)

	const fetchProtocols = useCallback(
		async (options?: GetAgentProtocolsOptions) => {
			setIsLoading(true)
			setError(null)
			try {
				const data = await client.getAgentProtocols(options)
				setProtocols(data.items)
				setNextCursor(data.next_cursor)
				return data
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

	const createProtocol = useCallback(
		async (payload: CreateAgentProtocolRequest) => {
			setError(null)
			try {
				const created = await client.createAgentProtocol(payload)
				setProtocols(prev => [...prev, created])
				return created
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	const updateProtocol = useCallback(
		async (id: string, payload: UpdateAgentProtocolRequest) => {
			setError(null)
			try {
				const updated = await client.updateAgentProtocol(id, payload)
				setProtocols(prev => prev.map(p => (p.id === id ? updated : p)))
				return updated
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	const deleteProtocol = useCallback(
		async (id: string) => {
			setError(null)
			try {
				await client.deleteAgentProtocol(id)
				setProtocols(prev => prev.filter(p => p.id !== id))
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	return {
		protocols,
		nextCursor,
		isLoading,
		error,
		fetchProtocols,
		createProtocol,
		updateProtocol,
		deleteProtocol,
	}
}
