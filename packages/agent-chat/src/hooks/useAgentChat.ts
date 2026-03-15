import { useCallback, useState } from 'react'
import { useAgentStream } from './useAgentStream'
import { useChatRoom } from './useChatRoom'

export function useAgentChat() {
	const [sessionId, setSessionId] = useState<string | null>(null)

	const stream = useAgentStream({
		sessionId,
		onSessionCreated: setSessionId,
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
		maxRequests: stream.maxRequests,
		setMaxRequests: stream.setMaxRequests,
		toolAccess: stream.toolAccess,
		setToolAccess: stream.setToolAccess,
		startTask: stream.startTask,
		sendUserMessage: stream.sendUserMessage,
		handleSubmit: stream.handleSubmit,
		resetChat: stream.resetChat,
		refetchMessages: stream.refetchMessages,

		// ChatRoom CRUD
		chatRooms: chatRoom.chatRooms,
		chatRoomLoading: chatRoom.isLoading,
		chatRoomError: chatRoom.error,
		fetchChatRooms: chatRoom.fetchChatRooms,
		createChatRoom: chatRoom.createChatRoom,
		updateChatRoom: chatRoom.updateChatRoom,
		deleteChatRoom: chatRoom.deleteChatRoom,
	}
}
