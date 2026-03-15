import { ChatPanel } from './ChatPanel'
import { FloatingChatPanel } from './FloatingChatPanel'

type AgentChatProps = {
	mode?: 'floating' | 'inline'
	className?: string
	panelWidth?: number
	panelHeight?: number
}

export function AgentChat({
	mode = 'floating',
	className,
	panelWidth,
	panelHeight,
}: AgentChatProps) {
	if (mode === 'inline') {
		return <ChatPanel className={className} />
	}
	return (
		<FloatingChatPanel
			className={className}
			panelWidth={panelWidth}
			panelHeight={panelHeight}
		/>
	)
}
