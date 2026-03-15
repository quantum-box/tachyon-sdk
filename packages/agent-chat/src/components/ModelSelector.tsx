import { useEffect, useState } from 'react'
import type { ModelInfo } from '../client/types'
import { useAgentChatContext } from '../providers/AgentChatProvider'

type ModelSelectorProps = {
	selectedModel: string | null
	onModelChange: (model: string) => void
	disabled?: boolean
	className?: string
}

export function ModelSelector({
	selectedModel,
	onModelChange,
	disabled,
	className,
}: ModelSelectorProps) {
	const { client } = useAgentChatContext()
	const [models, setModels] = useState<ModelInfo[]>([])

	useEffect(() => {
		let cancelled = false
		client
			.getModels()
			.then(m => {
				if (!cancelled) setModels(m)
			})
			.catch(() => {})
		return () => {
			cancelled = true
		}
	}, [client])

	return (
		<select
			value={selectedModel ?? ''}
			onChange={e => onModelChange(e.target.value)}
			disabled={disabled || models.length === 0}
			className={`w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm outline-none focus:border-blue-500 disabled:opacity-50 ${className ?? ''}`}
		>
			{models.length === 0 && <option value=''>Loading...</option>}
			{models.map(m => (
				<option key={m.id} value={m.id}>
					{m.name || m.id}
				</option>
			))}
		</select>
	)
}
