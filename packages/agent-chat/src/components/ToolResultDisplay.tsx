import type { AgentChunk } from '../client/types'

type ToolResultDisplayProps = {
	chunk: AgentChunk
}

export function ToolResultDisplay({ chunk }: ToolResultDisplayProps) {
	const result = chunk.tool_result || chunk.result || ''
	const isLong = result.length > 300
	return (
		<div className='flex justify-start mb-2'>
			<div className='max-w-[80%] rounded-xl px-3 py-2 bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 text-xs'>
				<span className='text-slate-600 dark:text-slate-400 font-medium'>
					{'\uD83D\uDCCB'} Result
				</span>
				<pre
					className={`mt-1 p-2 rounded bg-slate-100 dark:bg-slate-800 overflow-x-auto text-[11px] ${isLong ? 'max-h-48 overflow-y-auto' : ''}`}
				>
					{result}
				</pre>
			</div>
		</div>
	)
}
