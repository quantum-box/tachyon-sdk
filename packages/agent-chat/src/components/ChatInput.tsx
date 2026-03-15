import {
	type ChangeEvent,
	type FormEvent,
	type KeyboardEvent,
	useCallback,
	useRef,
} from 'react'

type ChatInputProps = {
	input: string
	onInputChange: (value: string) => void
	onSubmit: (e: FormEvent) => void
	isLoading?: boolean
	placeholder?: string
	className?: string
}

export function ChatInput({
	input,
	onInputChange,
	onSubmit,
	isLoading,
	placeholder,
	className,
}: ChatInputProps) {
	const textareaRef = useRef<HTMLTextAreaElement>(null)

	const handleKeyDown = useCallback(
		(e: KeyboardEvent<HTMLTextAreaElement>) => {
			if (e.key === 'Enter' && !e.shiftKey) {
				e.preventDefault()
				if (input.trim() && !isLoading) {
					onSubmit(e as unknown as FormEvent)
				}
			}
		},
		[input, isLoading, onSubmit],
	)

	const handleChange = useCallback(
		(e: ChangeEvent<HTMLTextAreaElement>) => {
			onInputChange(e.target.value)
			const el = e.target
			el.style.height = 'auto'
			el.style.height = `${Math.min(el.scrollHeight, 200)}px`
		},
		[onInputChange],
	)

	return (
		<form
			onSubmit={onSubmit}
			className={`border-t bg-white dark:bg-gray-900 p-3 ${className ?? ''}`}
		>
			<div className='flex items-end gap-2 max-w-4xl mx-auto'>
				<textarea
					ref={textareaRef}
					value={input}
					onChange={handleChange}
					onKeyDown={handleKeyDown}
					placeholder={placeholder ?? 'Type a message...'}
					disabled={isLoading}
					rows={1}
					className='flex-1 resize-none rounded-xl border border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800 px-4 py-2.5 text-sm outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 disabled:opacity-50 placeholder:text-gray-400'
				/>
				<button
					type='submit'
					disabled={!input.trim() || isLoading}
					className='shrink-0 rounded-xl bg-blue-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors'
				>
					{isLoading ? '...' : 'Send'}
				</button>
			</div>
		</form>
	)
}
