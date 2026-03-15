import { useEffect, useState } from 'react'
import { createPortal } from 'react-dom'
import { PERSISTENCE_KEYS, usePersisted } from '../hooks/useChatPersistence'
import { ChatPanel } from './ChatPanel'

type FloatingChatPanelProps = {
	className?: string
	panelWidth?: number
	panelHeight?: number
	buttonLabel?: string
}

export function FloatingChatPanel({
	className,
	panelWidth = 400,
	panelHeight = 600,
	buttonLabel,
}: FloatingChatPanelProps) {
	const [isOpen, setIsOpen] = usePersisted(PERSISTENCE_KEYS.panelOpen, false)
	const [sessionId, setSessionId] = useState<string | null>(null)
	const [mounted, setMounted] = useState(false)

	useEffect(() => {
		setMounted(true)
	}, [])

	useEffect(() => {
		const handler = (e: KeyboardEvent) => {
			if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
				e.preventDefault()
				setIsOpen((prev: boolean) => !prev)
			}
		}
		window.addEventListener('keydown', handler)
		return () => window.removeEventListener('keydown', handler)
	}, [setIsOpen])

	// Portal to document.body so that `position: fixed` is
	// always relative to the viewport, regardless of any
	// ancestor with `position: relative` / `transform` / etc.
	const content = (
		<>
			{isOpen && (
				<div
					className='fixed bottom-20 right-4 z-50 rounded-2xl shadow-2xl border border-gray-200 dark:border-gray-700 overflow-hidden'
					style={{ width: panelWidth, height: panelHeight }}
				>
					<div className='flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-800 border-b'>
						<span className='text-sm font-medium'>Agent Chat</span>
						<button
							type='button'
							onClick={() => setIsOpen(false)}
							className='text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 text-lg leading-none'
							aria-label='Close chat'
						>
							×
						</button>
					</div>
					<ChatPanel
						sessionId={sessionId}
						onSessionCreated={setSessionId}
						className='h-[calc(100%-40px)]'
					/>
				</div>
			)}

			<button
				type='button'
				onClick={() => setIsOpen((prev: boolean) => !prev)}
				className='fixed bottom-4 right-4 z-50 flex items-center justify-center w-14 h-14 rounded-full bg-blue-600 text-white shadow-lg hover:bg-blue-700 hover:shadow-xl transition-all'
				aria-label={buttonLabel ?? 'Open agent chat'}
			>
				{isOpen ? (
					<svg
						xmlns='http://www.w3.org/2000/svg'
						className='h-6 w-6'
						fill='none'
						viewBox='0 0 24 24'
						stroke='currentColor'
						strokeWidth={2}
					>
						<path
							strokeLinecap='round'
							strokeLinejoin='round'
							d='M6 18L18 6M6 6l12 12'
						/>
					</svg>
				) : (
					<svg
						xmlns='http://www.w3.org/2000/svg'
						className='h-6 w-6'
						fill='none'
						viewBox='0 0 24 24'
						stroke='currentColor'
						strokeWidth={2}
					>
						<path
							strokeLinecap='round'
							strokeLinejoin='round'
							d='M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z'
						/>
					</svg>
				)}
			</button>
		</>
	)

	if (!mounted) {
		return null
	}

	return createPortal(content, document.body)
}
