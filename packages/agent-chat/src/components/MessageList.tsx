import { useEffect, useMemo, useRef } from 'react'
import type { AgentChunk } from '../client/types'
import { MessageBubble } from './MessageBubble'

type MessageListProps = {
	chunks: AgentChunk[]
	isLoading?: boolean
	className?: string
}

/**
 * Merge consecutive chunks of the same type so that
 * streamed text deltas appear as a single message bubble.
 */
function mergeChunks(chunks: AgentChunk[]): AgentChunk[] {
	const merged: AgentChunk[] = []
	let current: AgentChunk | null = null

	for (const chunk of chunks) {
		switch (chunk.type) {
			case 'say':
			case 'assistant': {
				if (
					current &&
					(current.type === 'say' || current.type === 'assistant')
				) {
					current = {
						...current,
						text: (current.text ?? '') + (chunk.text ?? ''),
					} as AgentChunk
					merged[merged.length - 1] = current
				} else {
					current = { ...chunk } as AgentChunk
					merged.push(current)
				}
				continue
			}

			case 'thinking': {
				if (current?.type === 'thinking') {
					current = {
						...current,
						text: (current.text ?? '') + (chunk.text ?? ''),
						thinking:
							(current.thinking ?? '') + (chunk.thinking ?? ''),
						is_finished: chunk.is_finished,
					} as AgentChunk
					merged[merged.length - 1] = current
				} else {
					current = { ...chunk } as AgentChunk
					merged.push(current)
				}
				continue
			}

			case 'attempt_completion': {
				if (current?.type === 'attempt_completion') {
					current = {
						...current,
						result: (current.result ?? '') + (chunk.result ?? ''),
						command: chunk.command ?? current.command,
						is_finished: chunk.is_finished,
					} as AgentChunk
					merged[merged.length - 1] = current
				} else {
					current = { ...chunk } as AgentChunk
					merged.push(current)
				}
				continue
			}

			default: {
				current = chunk
				merged.push(chunk)
				continue
			}
		}
	}

	return merged
}

export function MessageList({
	chunks,
	isLoading,
	className,
}: MessageListProps) {
	const bottomRef = useRef<HTMLDivElement>(null)

	const mergedChunks = useMemo(() => mergeChunks(chunks), [chunks])

	useEffect(() => {
		bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
	}, [mergedChunks])

	return (
		<div
			className={`flex-1 overflow-y-auto p-4 ${className ?? ''}`}
			role='log'
			aria-live='polite'
		>
			{mergedChunks.length === 0 && !isLoading && (
				<div className='flex items-center justify-center h-full text-sm text-gray-400 dark:text-gray-500'>
					Send a message to start
				</div>
			)}
			{mergedChunks.map(chunk => (
				<MessageBubble key={chunk.id} chunk={chunk} />
			))}
			{isLoading && mergedChunks.length > 0 && (
				<div className='flex justify-start mb-2'>
					<div className='px-3 py-1.5 rounded-full bg-gray-100 dark:bg-gray-800 text-xs text-gray-500 animate-pulse'>
						Agent is working...
					</div>
				</div>
			)}
			<div ref={bottomRef} />
		</div>
	)
}
