import type { AgentChunk } from '../client/types'

type UsageSummaryProps = {
	chunk: AgentChunk
}

export function UsageSummary({ chunk }: UsageSummaryProps) {
	if (!chunk.total_tokens && !chunk.total_cost) return null
	return (
		<div className='flex justify-center mb-2'>
			<div className='inline-flex items-center gap-3 px-3 py-1 rounded-full bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700 text-[11px] text-gray-500 dark:text-gray-400'>
				{chunk.total_tokens != null && (
					<span>{chunk.total_tokens.toLocaleString()} tokens</span>
				)}
				{chunk.prompt_tokens != null && (
					<span>in: {chunk.prompt_tokens.toLocaleString()}</span>
				)}
				{chunk.completion_tokens != null && (
					<span>out: {chunk.completion_tokens.toLocaleString()}</span>
				)}
				{chunk.total_cost != null && (
					<span>${chunk.total_cost.toFixed(4)}</span>
				)}
			</div>
		</div>
	)
}
