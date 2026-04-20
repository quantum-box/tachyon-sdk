export type AgentChunk = {
  type:
    | 'user'
    | 'ask'
    | 'thinking'
    | 'tool_call'
    | 'tool_call_args'
    | 'tool_result'
    | 'tool_call_pending'
    | 'say'
    | 'attempt_completion'
    | 'assistant'
    | 'completion'
    | 'usage'
    | 'tool_job_started'
  id: string
  text?: string
  created_at: string
  user_id?: string
  thinking?: string
  tool_name?: string
  tool_arguments?: string
  tool_result?: string
  tool_id?: string
  args?: unknown
  content?: string
  result?: string
  command?: string | null
  index?: number
  is_finished?: boolean
  isStreaming?: boolean
  options?: string[]
  prompt_tokens?: number
  completion_tokens?: number
  total_tokens?: number
  total_cost?: number
  job_id?: string
  provider?: string
  agent?: { chatroom_id: string }
}

export type AgentExecuteRequest = {
  task: string
  agent_protocol_id?: string
  agent_protocol_mode?: 'disabled' | 'auto' | 'manual'
  user_custom_instructions?: string
  assistant_name?: string
  model?: string
  mcp_hub_config_json?: string
  additional_tool_description?: string
  auto_approve?: boolean
  max_requests?: number
  tool_access?: {
    filesystem?: boolean
    command?: boolean
    coding_agent_job?: boolean
    agent_protocol?: boolean
    web_search?: boolean
    url_fetch?: boolean
    sub_agent?: boolean
  }
}

export type ChatRoom = {
  id: string
  name: string
  created_at: string
}
