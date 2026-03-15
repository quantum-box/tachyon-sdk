import { useCallback, useState } from 'react'
import type { ChatRoom, CreateChatRoomResponse } from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'

export function useChatRoom() {
	const { client } = useAgentChatContext()
	const [chatRooms, setChatRooms] = useState<ChatRoom[]>([])
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<Error | null>(null)

	const fetchChatRooms = useCallback(async () => {
		setIsLoading(true)
		setError(null)
		try {
			const rooms = await client.getChatrooms()
			setChatRooms(rooms)
			return rooms
		} catch (e) {
			const err = e instanceof Error ? e : new Error(String(e))
			setError(err)
			throw err
		} finally {
			setIsLoading(false)
		}
	}, [client])

	const createChatRoom = useCallback(
		async (title?: string): Promise<CreateChatRoomResponse> => {
			setError(null)
			try {
				const res = await client.createChatRoom(title)
				setChatRooms(prev => [
					...prev,
					{
						id: res.chatroom.id,
						name: res.chatroom.name,
						created_at: new Date().toISOString(),
					},
				])
				return res
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	const updateChatRoom = useCallback(
		async (id: string, payload: { name?: string }) => {
			setError(null)
			try {
				const updated = await client.updateChatRoom(id, payload)
				setChatRooms(prev => prev.map(r => (r.id === id ? updated : r)))
				return updated
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	const deleteChatRoom = useCallback(
		async (id: string) => {
			setError(null)
			try {
				await client.deleteChatroom(id)
				setChatRooms(prev => prev.filter(r => r.id !== id))
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				throw err
			}
		},
		[client],
	)

	return {
		chatRooms,
		isLoading,
		error,
		fetchChatRooms,
		createChatRoom,
		updateChatRoom,
		deleteChatRoom,
	}
}
