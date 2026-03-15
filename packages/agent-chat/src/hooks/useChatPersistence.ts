import { useCallback, useState } from 'react'

function readStorage<T>(key: string, fallback: T): T {
	if (typeof window === 'undefined') return fallback
	try {
		const raw = localStorage.getItem(key)
		return raw ? (JSON.parse(raw) as T) : fallback
	} catch {
		return fallback
	}
}

function writeStorage(key: string, value: unknown): void {
	if (typeof window === 'undefined') return
	localStorage.setItem(key, JSON.stringify(value))
}

export function usePersisted<T>(key: string, fallback: T) {
	const [value, setValue] = useState<T>(() => readStorage(key, fallback))

	const update = useCallback(
		(next: T | ((prev: T) => T)) => {
			setValue(prev => {
				const resolved =
					typeof next === 'function' ? (next as (p: T) => T)(prev) : next
				writeStorage(key, resolved)
				return resolved
			})
		},
		[key],
	)

	return [value, update] as const
}

/** Convenience preset keys for agent-chat settings */
export const PERSISTENCE_KEYS = {
	maxRequests: 'agent-chat-max-requests',
	toolAccess: 'agent-chat-tool-access',
	selectedModel: 'agent-chat-selected-model',
	agentProtocolMode: 'agent-chat-protocol-mode',
	selectedProtocolId: 'agent-chat-selected-protocol-id',
	panelOpen: 'agent-chat-panel-open',
} as const
