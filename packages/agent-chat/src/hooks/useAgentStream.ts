import { useCallback, useEffect, useRef, useState } from 'react'
import type {
	AgentChunk,
	AgentExecuteRequest,
	ToolAccess,
} from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'
import { PERSISTENCE_KEYS, usePersisted } from './useChatPersistence'

export type UseAgentStreamOptions = {
	/** Current chatroom/session ID (controlled externally) */
	sessionId: string | null
	/** Callback when a new session is created */
	onSessionCreated?: (id: string) => void
}

export function useAgentStream(options: UseAgentStreamOptions) {
	const { client, config } = useAgentChatContext()
	const { sessionId, onSessionCreated } = options

	const [chunks, setChunks] = useState<AgentChunk[]>([])
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<Error | null>(null)
	const [input, setInput] = useState('')
	const isInitialLoadRef = useRef(true)
	const abortControllerRef = useRef<AbortController | null>(null)

	// Persisted settings
	const [selectedModel, setSelectedModel] = usePersisted<string | null>(
		PERSISTENCE_KEYS.selectedModel,
		config.defaultModel ?? 'anthropic/claude-sonnet-4.5',
	)
	const [maxRequests, setMaxRequests] = usePersisted<number>(
		PERSISTENCE_KEYS.maxRequests,
		10,
	)
	const [toolAccess, setToolAccess] = usePersisted<ToolAccess>(
		PERSISTENCE_KEYS.toolAccess,
		config.toolAccess ?? {
			filesystem: false,
			command: false,
			coding_agent_job: true,
			web_search: false,
			url_fetch: false,
			sub_agent: false,
		},
	)

	// Load initial messages when sessionId changes
	useEffect(() => {
		if (!sessionId) {
			setChunks([])
			isInitialLoadRef.current = true
			return
		}
		isInitialLoadRef.current = true
		let cancelled = false
		client
			.getMessages(sessionId)
			.then(messages => {
				if (cancelled) return
				setChunks(prev => {
					if (prev.length > 0) {
						isInitialLoadRef.current = false
						return prev
					}
					isInitialLoadRef.current = false
					return messages
				})
			})
			.catch((e: unknown) => {
				if (!cancelled) console.error('Failed to load messages:', e)
			})
		return () => {
			cancelled = true
		}
	}, [sessionId, client])

	// startTask: execute agent and stream SSE chunks
	const startTask = useCallback(
		async (task: string, newRoomTitle?: string) => {
			setIsLoading(true)
			setError(null)

			let currentSessionId = sessionId
			if (!currentSessionId) {
				try {
					const room = await client.createChatRoom(newRoomTitle)
					currentSessionId = room.chatroom.id
					onSessionCreated?.(currentSessionId)
				} catch (e) {
					const err = e instanceof Error ? e : new Error(String(e))
					setError(err)
					config.onError?.(err)
					setIsLoading(false)
					return
				}
			}

			if (abortControllerRef.current) abortControllerRef.current.abort()
			const ac = new AbortController()
			abortControllerRef.current = ac

			try {
				const args: AgentExecuteRequest = {
					task,
					model: selectedModel ?? undefined,
					max_requests: maxRequests,
					tool_access: toolAccess,
				}
				const response = await client.executeAgent(currentSessionId, args)
				if (!response.body) throw new Error('Response body is null')

				const reader = response.body.getReader()
				const decoder = new TextDecoder()
				let buffer = ''

				while (true) {
					if (ac.signal.aborted) break
					const { done, value } = await reader.read()
					if (done) break

					buffer += decoder.decode(value, {
						stream: true,
					})
					const lines = buffer.split('\n')
					buffer = lines.pop() ?? ''

					for (const line of lines) {
						if (!line.startsWith('data: ')) continue
						const data = line.slice(6).trim()
						if (!data || data === '[DONE]') continue
						try {
							const parsed = JSON.parse(data)
							if ('error' in parsed && parsed.error) {
								const msg =
									(
										parsed.error as {
											message?: string
										}
									).message ?? 'Unknown error'
								setError(new Error(msg))
								config.onError?.(new Error(msg))
								continue
							}
							const chunk = parsed as AgentChunk
							setChunks(prev => [...prev, chunk])
							config.onMessage?.(chunk)
						} catch {
							// skip malformed JSON
						}
					}
				}
			} catch (e) {
				if (e instanceof Error && e.name !== 'AbortError') {
					setError(e)
					config.onError?.(e)
				}
			} finally {
				setIsLoading(false)
				if (abortControllerRef.current === ac) abortControllerRef.current = null

				// Refetch to get server-persisted message IDs
				if (currentSessionId) {
					try {
						const msgs = await client.getMessages(currentSessionId)
						if (msgs.length > 0) setChunks(msgs)
					} catch {
						/* ignore */
					}
				}
			}
		},
		[
			sessionId,
			client,
			config,
			selectedModel,
			maxRequests,
			toolAccess,
			onSessionCreated,
		],
	)

	// sendUserMessage: add local user chunk then start task
	const sendUserMessage = useCallback(
		async (message: string) => {
			const trimmed = message.trim()
			if (!trimmed || isLoading) return false

			const userChunk: AgentChunk = {
				type: 'user',
				id: `user-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`,
				text: trimmed,
				created_at: new Date().toISOString(),
			}
			setChunks(prev => [...prev, userChunk])
			isInitialLoadRef.current = false
			await startTask(trimmed)
			return true
		},
		[isLoading, startTask],
	)

	const handleSubmit = useCallback(
		async (e: React.FormEvent) => {
			e.preventDefault()
			if (!input.trim() || isLoading) return
			const msg = input.trim()
			setInput('')
			await sendUserMessage(msg)
		},
		[input, isLoading, sendUserMessage],
	)

	const resetChat = useCallback(() => {
		if (abortControllerRef.current) {
			abortControllerRef.current.abort()
			abortControllerRef.current = null
		}
		setChunks([])
		setError(null)
		setIsLoading(false)
		setInput('')
		isInitialLoadRef.current = true
	}, [])

	const refetchMessages = useCallback(async () => {
		if (!sessionId) return
		try {
			const msgs = await client.getMessages(sessionId)
			setChunks(msgs)
		} catch {
			/* ignore */
		}
	}, [sessionId, client])

	// Cleanup on unmount
	useEffect(
		() => () => {
			abortControllerRef.current?.abort()
			abortControllerRef.current = null
		},
		[],
	)

	return {
		chunks,
		setChunks,
		isLoading,
		error,
		input,
		setInput,
		selectedModel,
		setSelectedModel,
		maxRequests,
		setMaxRequests,
		toolAccess,
		setToolAccess,
		startTask,
		sendUserMessage,
		handleSubmit,
		resetChat,
		refetchMessages,
	}
}
