import { ChatPanel } from './ChatPanel'
import {
	FloatingChatPanel,
	type AgentChatTheme,
	type FloatingChatPanelPosition,
} from './FloatingChatPanel'

export type AgentChatProps = {
	mode?: 'floating' | 'inline'
	className?: string
	style?: React.CSSProperties
	panelWidth?: number
	panelHeight?: number
	buttonLabel?: string
	zIndex?: number
	position?: FloatingChatPanelPosition
	theme?: AgentChatTheme
}

export function AgentChat({
	mode = 'floating',
	className,
	style,
	panelWidth,
	panelHeight,
	buttonLabel,
	zIndex,
	position,
	theme,
}: AgentChatProps) {
	if (mode === 'inline') {
		return <ChatPanel className={className} />
	}
	return (
		<FloatingChatPanel
			className={className}
			style={style}
			panelWidth={panelWidth}
			panelHeight={panelHeight}
			buttonLabel={buttonLabel}
			zIndex={zIndex}
			position={position}
			theme={theme}
		/>
	)
}
