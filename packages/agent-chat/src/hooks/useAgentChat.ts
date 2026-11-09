import { useCallback, useState } from 'react'
import { useAgentSessions } from './useAgentSessions'
import { useAgentStream } from './useAgentStream'
import { useChatRoom } from './useChatRoom'

export function useAgentChat() {
	const [sessionId, setSessionId] = useState<string | null>(null)

	const sessions = useAgentSessions()
	const onSessionCreated = useCallback(
		(id: string) => {
			setSessionId(id)
			void sessions.fetchSessions().catch(() => {
				/* exposed through sessionsError */
			})
		},
		[sessions.fetchSessions],
	)

	const stream = useAgentStream({
		sessionId,
		onSessionCreated,
	})

	const chatRoom = useChatRoom()

	const newChat = useCallback(() => {
		stream.resetChat()
		setSessionId(null)
	}, [stream])

	const selectSession = useCallback(
		async (id: string) => {
			stream.resetChat()
			setSessionId(id)
		},
		[stream],
	)

	return {
		// Session
		sessionId,
		setSessionId,
		newChat,
		selectSession,

		// Stream (forwarded)
		chunks: stream.chunks,
		setChunks: stream.setChunks,
		isLoading: stream.isLoading,
		error: stream.error,
		input: stream.input,
		setInput: stream.setInput,
		selectedModel: stream.selectedModel,
		setSelectedModel: stream.setSelectedModel,
		availableModels: stream.availableModels,
		modelsLoading: stream.modelsLoading,
		modelsError: stream.modelsError,
		maxRequests: stream.maxRequests,
		setMaxRequests: stream.setMaxRequests,
		toolAccess: stream.toolAccess,
		setToolAccess: stream.setToolAccess,
		startTask: stream.startTask,
		sendUserMessage: stream.sendUserMessage,
		handleSubmit: stream.handleSubmit,
		resetChat: stream.resetChat,
		refetchMessages: stream.refetchMessages,

		// Canonical conversation objects
		sessions: sessions.sessions,
		sessionsLoading: sessions.isLoading,
		sessionsError: sessions.error,
		fetchSessions: sessions.fetchSessions,
		createSession: sessions.createSession,
		deleteSession: sessions.deleteSession,

		// Deprecated compatibility for callers managing legacy ChatRooms.
		chatRooms: chatRoom.chatRooms,
		chatRoomLoading: chatRoom.isLoading,
		chatRoomError: chatRoom.error,
		fetchChatRooms: chatRoom.fetchChatRooms,
		createChatRoom: chatRoom.createChatRoom,
		updateChatRoom: chatRoom.updateChatRoom,
		deleteChatRoom: chatRoom.deleteChatRoom,
	}
}
