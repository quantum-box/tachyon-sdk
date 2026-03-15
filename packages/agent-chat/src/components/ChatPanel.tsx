import { useState } from 'react'
import { useAgentStream } from '../hooks/useAgentStream'
import { ChatInput } from './ChatInput'
import { MessageList } from './MessageList'
import { ModelSelector } from './ModelSelector'

type ChatPanelProps = {
	sessionId?: string | null
	onSessionCreated?: (id: string) => void
	showModelSelector?: boolean
	className?: string
}

export function ChatPanel({
	sessionId: externalSessionId,
	onSessionCreated,
	showModelSelector = true,
	className,
}: ChatPanelProps) {
	const [internalSessionId, setInternalSessionId] = useState<string | null>(
		externalSessionId ?? null,
	)
	const effectiveSessionId =
		externalSessionId !== undefined ? externalSessionId : internalSessionId

	const stream = useAgentStream({
		sessionId: effectiveSessionId ?? null,
		onSessionCreated: id => {
			setInternalSessionId(id)
			onSessionCreated?.(id)
		},
	})

	return (
		<div
			className={`flex flex-col bg-white dark:bg-gray-900 ${className ?? 'h-full'}`}
		>
			{showModelSelector && (
				<div className='flex items-center gap-2 px-3 py-2 border-b'>
					<ModelSelector
						selectedModel={stream.selectedModel}
						onModelChange={stream.setSelectedModel}
						disabled={stream.isLoading}
						className='max-w-xs'
					/>
				</div>
			)}
			{stream.error && (
				<div className='mx-3 mt-2 px-3 py-2 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-xs text-red-700 dark:text-red-300'>
					{stream.error.message}
				</div>
			)}
			<MessageList
				chunks={stream.chunks}
				isLoading={stream.isLoading}
				className='flex-1'
			/>
			<ChatInput
				input={stream.input}
				onInputChange={stream.setInput}
				onSubmit={stream.handleSubmit}
				isLoading={stream.isLoading}
			/>
		</div>
	)
}
