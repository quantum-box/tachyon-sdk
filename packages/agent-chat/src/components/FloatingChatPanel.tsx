import { useEffect, useState } from 'react'
import { createPortal } from 'react-dom'
import { PERSISTENCE_KEYS, usePersisted } from '../hooks/useChatPersistence'
import { ChatPanel } from './ChatPanel'

export type AgentChatTheme = {
	primary?: string
	primaryHover?: string
	background?: string
	headerBackground?: string
	border?: string
}

export type FloatingChatPanelPosition =
	| 'bottom-right'
	| 'bottom-left'
	| 'top-right'
	| 'top-left'

export type FloatingChatPanelProps = {
	className?: string
	style?: React.CSSProperties
	panelWidth?: number
	panelHeight?: number
	buttonLabel?: string
	zIndex?: number
	position?: FloatingChatPanelPosition
	theme?: AgentChatTheme
}

const POSITION_MAP: Record<
	FloatingChatPanelPosition,
	{ button: string; panel: string }
> = {
	'bottom-right': { button: 'bottom-4 right-4', panel: 'bottom-20 right-4' },
	'bottom-left': { button: 'bottom-4 left-4', panel: 'bottom-20 left-4' },
	'top-right': { button: 'top-4 right-4', panel: 'top-20 right-4' },
	'top-left': { button: 'top-4 left-4', panel: 'top-20 left-4' },
}

export function FloatingChatPanel({
	className,
	style,
	panelWidth = 400,
	panelHeight = 600,
	buttonLabel,
	zIndex = 50,
	position = 'bottom-right',
	theme,
}: FloatingChatPanelProps) {
	const [isOpen, setIsOpen] = usePersisted(PERSISTENCE_KEYS.panelOpen, false)
	const [sessionId, setSessionId] = useState<string | null>(null)
	const [mounted, setMounted] = useState(false)
	const [isButtonHovered, setIsButtonHovered] = useState(false)

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

	const pos = POSITION_MAP[position]

	const cssVars = {
		...(theme?.primary != null && { '--agentchat-primary': theme.primary }),
		...(theme?.primaryHover != null && {
			'--agentchat-primary-hover': theme.primaryHover,
		}),
		...(theme?.background != null && { '--agentchat-bg': theme.background }),
		...(theme?.headerBackground != null && {
			'--agentchat-header-bg': theme.headerBackground,
		}),
		...(theme?.border != null && { '--agentchat-border': theme.border }),
	} as React.CSSProperties

	const content = (
		<div className={className} style={{ ...cssVars, ...style }}>
			{isOpen && (
				<div
					className={`fixed ${pos.panel} rounded-2xl shadow-2xl overflow-hidden`}
					style={{
						width: panelWidth,
						height: panelHeight,
						zIndex,
						borderWidth: 1,
						borderStyle: 'solid',
						borderColor: 'var(--agentchat-border, rgb(229 231 235))',
					}}
				>
					<div
						className='flex items-center justify-between px-4 py-2 border-b'
						style={{
							backgroundColor: 'var(--agentchat-header-bg, rgb(249 250 251))',
							borderColor: 'var(--agentchat-border, rgb(229 231 235))',
						}}
					>
						<span className='text-sm font-medium'>Agent Chat</span>
						<button
							type='button'
							onClick={() => setIsOpen(false)}
							className='text-gray-400 hover:text-gray-600 text-lg leading-none'
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
				className={`fixed ${pos.button} flex items-center justify-center w-14 h-14 rounded-full text-white shadow-lg hover:shadow-xl transition-all`}
				style={{
					zIndex,
					backgroundColor: isButtonHovered
						? 'var(--agentchat-primary-hover, #1d4ed8)'
						: 'var(--agentchat-primary, #2563eb)',
				}}
				onMouseEnter={() => setIsButtonHovered(true)}
				onMouseLeave={() => setIsButtonHovered(false)}
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
		</div>
	)

	if (!mounted) {
		return null
	}

	return createPortal(content, document.body)
}
