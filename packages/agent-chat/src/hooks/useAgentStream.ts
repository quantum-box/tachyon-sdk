import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type {
	AgentChunk,
	AgentExecuteRequest,
	ClientToolDefinition,
	ModelInfo,
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
	const [availableModels, setAvailableModels] = useState<ModelInfo[]>([])
	const [modelsLoading, setModelsLoading] = useState(true)
	const [modelsError, setModelsError] = useState<Error | null>(null)
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
	const [rawToolAccess, setRawToolAccess] = usePersisted<unknown>(
		PERSISTENCE_KEYS.toolAccess,
		normalizeToolAccess(config.toolAccess),
	)
	const toolAccess = useMemo(
		() => normalizeToolAccess(rawToolAccess),
		[rawToolAccess],
	)
	const setToolAccess = useCallback(
		(next: ToolAccess | ((prev: ToolAccess) => ToolAccess)) => {
			setRawToolAccess((prev: unknown) => {
				const current = normalizeToolAccess(prev)
				return typeof next === 'function'
					? next(current)
					: normalizeToolAccess(next)
			})
		},
		[setRawToolAccess],
	)

	// Keep model availability in the same hook that performs execution so
	// persisted selections cannot bypass the dropdown.
	useEffect(() => {
		let cancelled = false
		setModelsLoading(true)
		setModelsError(null)
		client
			.getModels()
			.then(models => {
				if (cancelled) return
				setAvailableModels(models)
				setSelectedModel(current => coerceSelectedModel(current, models))
			})
			.catch((e: unknown) => {
				if (cancelled) return
				const err = e instanceof Error ? e : new Error(String(e))
				setAvailableModels([])
				setModelsError(err)
			})
			.finally(() => {
				if (!cancelled) setModelsLoading(false)
			})
		return () => {
			cancelled = true
		}
	}, [client, setSelectedModel])

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

	const handlePendingClientTool = useCallback(
		async (currentSessionId: string, chunk: AgentChunk) => {
			const toolName = chunk.tool_name
			const toolId = chunk.tool_id
			if (!toolName || !toolId) return

			const args = getToolArgs(chunk)
			const handler =
				toolName === 'canvas'
					? (config.onCanvasToolCall ??
						config.clientToolHandlers?.canvas ??
						dispatchCanvasTool)
					: config.clientToolHandlers?.[toolName]

			if (!handler) {
				if (toolName === 'canvas') {
					const message =
						'Canvas tool was requested, but no canvas renderer is configured.'
					setChunks(prev => [...prev, localErrorChunk(message)])
					if (!chunk.fire_and_forget) {
						try {
							await client.submitToolResult(currentSessionId, {
								tool_id: toolId,
								result: message,
								is_error: true,
							})
						} catch (e) {
							const err = e instanceof Error ? e : new Error(String(e))
							setError(err)
							config.onError?.(err)
							setChunks(prev => [...prev, localErrorChunk(err.message)])
						}
					}
				}
				return
			}

			try {
				const result = await handler(args, chunk)
				if (!chunk.fire_and_forget) {
					try {
						await client.submitToolResult(currentSessionId, {
							tool_id: toolId,
							result: stringifyToolResult(result),
							is_error: false,
						})
					} catch (e) {
						const err = e instanceof Error ? e : new Error(String(e))
						setError(err)
						config.onError?.(err)
						setChunks(prev => [...prev, localErrorChunk(err.message)])
					}
				}
			} catch (e) {
				const err = e instanceof Error ? e : new Error(String(e))
				setError(err)
				config.onError?.(err)
				setChunks(prev => [...prev, localErrorChunk(err.message)])
				if (!chunk.fire_and_forget) {
					try {
						await client.submitToolResult(currentSessionId, {
							tool_id: toolId,
							result: err.message,
							is_error: true,
						})
					} catch (submitError) {
						const submitErr =
							submitError instanceof Error
								? submitError
								: new Error(String(submitError))
						config.onError?.(submitErr)
						setChunks(prev => [...prev, localErrorChunk(submitErr.message)])
					}
				}
			}
		},
		[client, config],
	)

	// startTask: execute agent and stream SSE chunks
	const startTask = useCallback(
		async (task: string, newRoomTitle?: string) => {
			const modelGuardError = validateSelectedModel(
				selectedModel,
				availableModels,
				modelsLoading,
				modelsError,
			)
			if (modelGuardError) {
				const localError = localErrorChunk(modelGuardError.message)
				setChunks(prev => [...prev, localError])
				setError(modelGuardError)
				config.onError?.(modelGuardError)
				return
			}

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
				const clientTools = buildClientTools(config)
				const args: AgentExecuteRequest = {
					task,
					model: selectedModel ?? undefined,
					max_requests: maxRequests,
					tool_access: toolAccess,
					client_tools: clientTools.length > 0 ? clientTools : undefined,
					use_json_tool_calls: clientTools.length > 0 ? true : undefined,
					user_custom_instructions:
						config.enableCanvasTool === false
							? undefined
							: buildCanvasInstructions(),
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
							if (chunk.type === 'tool_call_pending') {
								await handlePendingClientTool(currentSessionId, chunk)
							}
						} catch {
							// skip malformed JSON
						}
					}
				}
			} catch (e) {
				if (e instanceof Error && e.name !== 'AbortError') {
					setError(e)
					config.onError?.(e)
					setChunks(prev => [...prev, localErrorChunk(e.message)])
				}
			} finally {
				setIsLoading(false)
				if (abortControllerRef.current === ac) abortControllerRef.current = null

				// Refetch to get server-persisted message IDs
				if (currentSessionId) {
					try {
						const msgs = await client.getMessages(currentSessionId)
						if (msgs.length > 0) {
							setChunks(reconcileCanvasPromiseWarning(msgs))
						}
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
			availableModels,
			modelsLoading,
			modelsError,
			maxRequests,
			toolAccess,
			handlePendingClientTool,
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
		availableModels,
		modelsLoading,
		modelsError,
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

function coerceSelectedModel(
	current: string | null,
	models: ModelInfo[],
): string | null {
	if (models.length === 0) return null
	if (current && models.some(model => model.id === current)) return current
	return models[0]?.id ?? null
}

function validateSelectedModel(
	selectedModel: string | null,
	models: ModelInfo[],
	modelsLoading: boolean,
	modelsError: Error | null,
): Error | null {
	if (modelsLoading) {
		return new Error('Model list is still loading. Please try again shortly.')
	}
	if (modelsError) {
		return new Error(
			`Unable to load available models. Please refresh and try again. (${modelsError.message})`,
		)
	}
	if (models.length === 0) {
		return new Error(
			'No executable agent models are available for this tenant.',
		)
	}
	if (!selectedModel || !models.some(model => model.id === selectedModel)) {
		return new Error(
			'The selected model is not available. Please select an available model and try again.',
		)
	}
	return null
}

function buildClientTools(config: {
	clientTools?: ClientToolDefinition[]
	enableCanvasTool?: boolean
}): ClientToolDefinition[] {
	const tools = config.clientTools ?? []
	if (config.enableCanvasTool === false) return tools
	if (tools.some(tool => tool.name === CANVAS_TOOL.name)) return tools
	return [CANVAS_TOOL, ...tools]
}

const CANVAS_TOOL: ClientToolDefinition = {
	name: 'canvas',
	description:
		'Render or update a visible canvas/artifact for the user. Use this tool whenever you say that content will be shown on the canvas, displayed as an artifact, or opened visually.',
	parameters: {
		type: 'object',
		properties: {
			title: {
				type: 'string',
				description: 'Short title for the canvas.',
			},
			content: {
				type: 'string',
				description: 'Markdown, HTML, SVG, or text content to display.',
			},
			content_type: {
				type: 'string',
				description:
					'MIME type or content kind such as text/markdown, text/html, image/svg+xml, or text/plain.',
			},
			language: {
				type: 'string',
				description: 'Optional language identifier for code or markdown.',
			},
		},
		required: ['content'],
		additionalProperties: true,
	},
	fire_and_forget: false,
}

function buildCanvasInstructions(): string {
	return [
		'Canvas client tool policy:',
		'- If you say that something will be displayed, opened, rendered, or shown on the canvas, call the `canvas` client tool in the same turn.',
		'- Do not say "キャンバスで表示します" or similar future-tense canvas promises unless you actually invoke the `canvas` tool.',
		'- When the user asks for a canvas, artifact, visual summary, diagram, table, or document-style output, use the `canvas` tool instead of only describing the output in chat.',
	].join('\n')
}

function normalizeToolAccess(toolAccess: unknown): ToolAccess {
	if (Array.isArray(toolAccess)) {
		return toolAccess.filter(
			(tool): tool is ToolAccess[number] =>
				typeof tool === 'object' &&
				tool !== null &&
				(tool as { type?: unknown }).type === 'builtin' &&
				typeof (tool as { name?: unknown }).name === 'string',
		)
	}

	if (toolAccess && typeof toolAccess === 'object') {
		return Object.entries(toolAccess)
			.filter(([, enabled]) => enabled === true)
			.map(([name]) => ({ type: 'builtin', name }))
	}

	return []
}

function getToolArgs(chunk: AgentChunk): unknown {
	if (chunk.args !== undefined) return chunk.args
	if (!chunk.tool_arguments) return {}
	try {
		return JSON.parse(chunk.tool_arguments)
	} catch {
		return { raw: chunk.tool_arguments }
	}
}

function stringifyToolResult(result: unknown): string {
	if (result === undefined) return JSON.stringify({ ok: true })
	if (typeof result === 'string') return result
	return JSON.stringify(result)
}

function dispatchCanvasTool(
	args: unknown,
	chunk: AgentChunk,
): Record<string, unknown> {
	if (typeof window === 'undefined') {
		throw new Error('Canvas tool is only available in a browser runtime.')
	}
	window.dispatchEvent(
		new CustomEvent('tachyon:agent-canvas', {
			detail: {
				args,
				chunk,
			},
		}),
	)
	return {
		ok: true,
		message: 'Canvas event dispatched to the host application.',
	}
}

function localErrorChunk(message: string): AgentChunk {
	return {
		type: 'error',
		id: `local-error-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`,
		created_at: new Date().toISOString(),
		message,
		text: message,
	}
}

function reconcileCanvasPromiseWarning(messages: AgentChunk[]): AgentChunk[] {
	if (!mentionsCanvasPromise(messages)) return messages
	if (messages.some(isCanvasOrArtifactChunk)) return messages
	return [
		...messages,
		localErrorChunk(
			'The assistant promised a canvas/artifact, but no canvas tool call or artifact was produced. Please retry and ask it to use the canvas tool.',
		),
	]
}

function mentionsCanvasPromise(messages: AgentChunk[]): boolean {
	return messages.some(message => {
		if (
			message.type !== 'say' &&
			message.type !== 'assistant' &&
			message.type !== 'attempt_completion'
		) {
			return false
		}
		const text = message.text ?? message.content ?? message.result ?? ''
		return /キャンバス|canvas|artifact|アーティファクト/.test(text)
	})
}

function isCanvasOrArtifactChunk(message: AgentChunk): boolean {
	return (
		message.type === 'artifact' ||
		((message.type === 'tool_call' ||
			message.type === 'tool_call_pending' ||
			message.type === 'tool_call_args') &&
			message.tool_name === 'canvas')
	)
}
