import { useState, useRef } from 'react'
import { streamAgent } from '@quantumbox/tachyon-agent'

const API_KEY = import.meta.env.VITE_TACHYON_API_KEY as string
const BASE_URL = (import.meta.env.VITE_TACHYON_API_URL as string) ?? 'https://api.n1.tachy.one'

type Message = {
  role: 'user' | 'assistant'
  content: string
  streaming?: boolean
}

export default function App() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const abortRef = useRef<AbortController | null>(null)

  const send = async () => {
    const prompt = input.trim()
    if (!prompt || loading) return

    setInput('')
    setMessages((prev) => [...prev, { role: 'user', content: prompt }])
    setLoading(true)

    // Add empty assistant message that will be filled by streaming
    setMessages((prev) => [
      ...prev,
      { role: 'assistant', content: '', streaming: true },
    ])

    abortRef.current = new AbortController()

    try {
      for await (const event of streamAgent(prompt, { apiKey: API_KEY, baseUrl: BASE_URL })) {
        if (event.type === 'text') {
          setMessages((prev) => {
            const next = [...prev]
            const last = next[next.length - 1]
            if (last?.role === 'assistant') {
              next[next.length - 1] = { ...last, content: last.content + event.content }
            }
            return next
          })
        }
        if (event.type === 'done') break
      }
    } catch (e) {
      console.error(e)
    } finally {
      setMessages((prev) => {
        const next = [...prev]
        const last = next[next.length - 1]
        if (last?.role === 'assistant') {
          next[next.length - 1] = { ...last, streaming: false }
        }
        return next
      })
      setLoading(false)
    }
  }

  return (
    <div style={{ maxWidth: 720, margin: '0 auto', padding: '1rem', fontFamily: 'sans-serif' }}>
      <h1 style={{ fontSize: '1.25rem', marginBottom: '1rem' }}>
        Tachyon Agent Chat
      </h1>

      <div
        style={{
          border: '1px solid #e5e7eb',
          borderRadius: 8,
          minHeight: 400,
          maxHeight: 600,
          overflowY: 'auto',
          padding: '1rem',
          marginBottom: '0.75rem',
          background: '#fafafa',
        }}
      >
        {messages.length === 0 && (
          <p style={{ color: '#9ca3af', textAlign: 'center', marginTop: 160 }}>
            メッセージを送信してください
          </p>
        )}
        {messages.map((msg, i) => (
          <div
            key={i}
            style={{
              marginBottom: '0.75rem',
              display: 'flex',
              flexDirection: msg.role === 'user' ? 'row-reverse' : 'row',
              gap: '0.5rem',
              alignItems: 'flex-start',
            }}
          >
            <div
              style={{
                background: msg.role === 'user' ? '#3b82f6' : '#ffffff',
                color: msg.role === 'user' ? '#ffffff' : '#111827',
                border: msg.role === 'assistant' ? '1px solid #e5e7eb' : 'none',
                borderRadius: 8,
                padding: '0.5rem 0.75rem',
                maxWidth: '80%',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
              }}
            >
              {msg.content}
              {msg.streaming && <span style={{ opacity: 0.5 }}>▋</span>}
            </div>
          </div>
        ))}
      </div>

      <div style={{ display: 'flex', gap: '0.5rem' }}>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && send()}
          placeholder="メッセージを入力..."
          disabled={loading}
          style={{
            flex: 1,
            padding: '0.5rem 0.75rem',
            border: '1px solid #d1d5db',
            borderRadius: 6,
            fontSize: '0.95rem',
          }}
        />
        <button
          onClick={send}
          disabled={loading || !input.trim()}
          style={{
            padding: '0.5rem 1.25rem',
            background: '#3b82f6',
            color: '#fff',
            border: 'none',
            borderRadius: 6,
            cursor: loading ? 'not-allowed' : 'pointer',
            opacity: loading || !input.trim() ? 0.5 : 1,
          }}
        >
          送信
        </button>
      </div>

      {!API_KEY && (
        <p style={{ color: '#ef4444', marginTop: '0.5rem', fontSize: '0.85rem' }}>
          ⚠️ VITE_TACHYON_API_KEY が未設定です。.env ファイルを確認してください。
        </p>
      )}
    </div>
  )
}
