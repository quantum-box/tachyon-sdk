type ThinkingIndicatorProps = {
	text?: string
	isFinished?: boolean
}

export function ThinkingIndicator({
	text,
	isFinished,
}: ThinkingIndicatorProps) {
	return (
		<div className='flex justify-start mb-2'>
			<div className='max-w-[80%] rounded-xl px-3 py-2 bg-purple-50 dark:bg-purple-900/20 border border-purple-200 dark:border-purple-800 text-xs text-purple-700 dark:text-purple-300'>
				<div className='flex items-center gap-1.5 mb-1'>
					<span className='font-medium'>Thinking</span>
					{!isFinished && (
						<span className='inline-flex gap-0.5'>
							<span className='animate-bounce [animation-delay:-0.3s]'>.</span>
							<span className='animate-bounce [animation-delay:-0.15s]'>.</span>
							<span className='animate-bounce'>.</span>
						</span>
					)}
				</div>
				{text && (
					<p className='whitespace-pre-wrap opacity-80 line-clamp-6'>{text}</p>
				)}
			</div>
		</div>
	)
}
