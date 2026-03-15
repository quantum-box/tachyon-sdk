import type { AgentChunk } from '../client/types'

type ToolCallDisplayProps = {
	chunk: AgentChunk
}

export function ToolCallDisplay({ chunk }: ToolCallDisplayProps) {
	const isPending = chunk.type === 'tool_call_pending'
	return (
		<div className='flex justify-start mb-2'>
			<div className='max-w-[80%] rounded-xl px-3 py-2 bg-emerald-50 dark:bg-emerald-900/20 border border-emerald-200 dark:border-emerald-800 text-xs'>
				<div className='flex items-center gap-1.5'>
					<span className='text-emerald-600 dark:text-emerald-400 font-medium'>
						{isPending ? '\u23F3' : '\uD83D\uDD27'}{' '}
						{chunk.tool_name || 'Tool Call'}
					</span>
					{isPending && (
						<span className='text-emerald-500 animate-pulse'>running...</span>
					)}
				</div>
				{chunk.tool_arguments && (
					<pre className='mt-1 p-2 rounded bg-emerald-100/50 dark:bg-emerald-900/30 overflow-x-auto text-[11px] max-h-32 overflow-y-auto'>
						{chunk.tool_arguments}
					</pre>
				)}
			</div>
		</div>
	)
}
