import type { AgentChunk } from '../client/types'
import { ThinkingIndicator } from './ThinkingIndicator'
import { ToolCallDisplay } from './ToolCallDisplay'
import { ToolResultDisplay } from './ToolResultDisplay'
import { UsageSummary } from './UsageSummary'

type MessageBubbleProps = {
	chunk: AgentChunk
	isStreaming?: boolean
}

export function MessageBubble({ chunk }: MessageBubbleProps) {
	switch (chunk.type) {
		case 'user':
			return (
				<div className='flex justify-end mb-3'>
					<div className='max-w-[80%] rounded-2xl rounded-br-md px-4 py-2.5 bg-blue-600 text-white text-sm whitespace-pre-wrap'>
						{chunk.text}
					</div>
				</div>
			)
		case 'say':
		case 'assistant':
			return (
				<div className='flex justify-start mb-3'>
					<div className='max-w-[80%] rounded-2xl rounded-bl-md px-4 py-2.5 bg-gray-100 dark:bg-gray-800 text-sm whitespace-pre-wrap'>
						{chunk.text || chunk.content || ''}
					</div>
				</div>
			)
		case 'attempt_completion':
			return (
				<div className='flex justify-start mb-3'>
					<div className='max-w-[80%] rounded-2xl rounded-bl-md px-4 py-2.5 bg-gray-100 dark:bg-gray-800 text-sm whitespace-pre-wrap'>
						{chunk.result || chunk.text || chunk.content || ''}
					</div>
				</div>
			)
		case 'thinking':
			return (
				<ThinkingIndicator
					text={chunk.thinking || chunk.text}
					isFinished={chunk.is_finished}
				/>
			)
		case 'tool_call':
		case 'tool_call_args':
		case 'tool_call_pending':
			return <ToolCallDisplay chunk={chunk} />
		case 'tool_result':
			return <ToolResultDisplay chunk={chunk} />
		case 'usage':
			return <UsageSummary chunk={chunk} />
		case 'tool_job_started':
			return (
				<div className='flex justify-start mb-2'>
					<div className='text-xs px-3 py-1.5 rounded-full bg-amber-50 dark:bg-amber-900/20 text-amber-700 dark:text-amber-300 border border-amber-200 dark:border-amber-800'>
						Tool Job started: {chunk.provider || 'unknown'} (
						{chunk.job_id?.slice(0, 8)}...)
					</div>
				</div>
			)
		case 'ask':
			return (
				<div className='flex justify-start mb-3'>
					<div className='max-w-[80%] rounded-2xl rounded-bl-md px-4 py-2.5 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 text-sm'>
						<p className='font-medium mb-2'>{chunk.text}</p>
						{chunk.options && (
							<div className='flex flex-wrap gap-2'>
								{chunk.options.map(opt => (
									<span
										key={opt}
										className='px-2 py-1 text-xs rounded bg-yellow-100 dark:bg-yellow-800/30'
									>
										{opt}
									</span>
								))}
							</div>
						)}
					</div>
				</div>
			)
		default:
			return null
	}
}
