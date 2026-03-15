import { useCallback, useState } from 'react'
import type { CreateMemoryRequest, SavedMemory } from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'

export function useSavedMemory() {
	const { client } = useAgentChatContext()
	const [memories, setMemories] = useState<SavedMemory[]>([])
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<Error | null>(null)

	const fetchMemories = useCallback(
		async (status?: 'ACTIVE' | 'ARCHIVED') => {
			setIsLoading(true)
			setError(null)
			try {
				const result = await client.getMemories(status)
				setMemories(result)
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

	const createMemory = useCallback(
		async (payload: CreateMemoryRequest) => {
			setError(null)
			try {
				const created = await client.createMemory(payload)
				setMemories(prev => [...prev, created])
				return created
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	const deleteMemory = useCallback(
		async (id: string) => {
			setError(null)
			try {
				await client.deleteMemory(id)
				setMemories(prev => prev.filter(m => m.id !== id))
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	const archiveMemory = useCallback(
		async (id: string) => {
			setError(null)
			try {
				const updated = await client.archiveMemory(id)
				setMemories(prev => prev.map(m => (m.id === id ? updated : m)))
				return updated
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	const activateMemory = useCallback(
		async (id: string) => {
			setError(null)
			try {
				const updated = await client.activateMemory(id)
				setMemories(prev => prev.map(m => (m.id === id ? updated : m)))
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
		memories,
		isLoading,
		error,
		fetchMemories,
		createMemory,
		deleteMemory,
		archiveMemory,
		activateMemory,
	}
}
