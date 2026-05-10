import type { ModelInfo } from '../client/types'

type ModelSelectorProps = {
	selectedModel: string | null
	onModelChange: (model: string) => void
	models: ModelInfo[]
	isLoading?: boolean
	error?: Error | null
	disabled?: boolean
	className?: string
}

export function ModelSelector({
	selectedModel,
	onModelChange,
	models,
	isLoading,
	error,
	disabled,
	className,
}: ModelSelectorProps) {
	const hasModels = models.length > 0
	return (
		<select
			value={selectedModel ?? ''}
			onChange={e => onModelChange(e.target.value)}
			disabled={disabled || !hasModels}
			className={`w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm outline-none focus:border-blue-500 disabled:opacity-50 ${className ?? ''}`}
		>
			{!hasModels && (
				<option value=''>
					{isLoading
						? 'Loading models...'
						: error
							? 'Models unavailable'
							: 'No models available'}
				</option>
			)}
			{models.map(m => (
				<option key={m.id} value={m.id}>
					{m.name || m.id}
				</option>
			))}
		</select>
	)
}
