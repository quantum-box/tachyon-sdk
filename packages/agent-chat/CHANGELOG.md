# Changelog

All notable changes to `@anthropic-ja/agent-chat` will be documented in this file.

## [0.2.2] - 2026-04-17

### Added
- `skills/image-gen.json`: Anthropic tool-schema definition for `tachyon image generate`, bundled in the npm package and importable via `@anthropic-ja/agent-chat/skills/image-gen.json`
- Support for multiple image generation models (gpt-image-1.5, gpt-image-1, gpt-image-1-mini, grok-2-image, gemini-2.0-flash-exp-image-generation)

## [0.2.0] - 2026-03-15

### Changed
- Package restructured as npm workspace for git-dependency support (monorepo integration)

## [0.1.0] - 2026-03-15

### Added
- `AgentChatClient`: Full API client covering chat, memory, protocols, insights, tool jobs, and tool search endpoints
- React components: `AgentChat`, `ChatPanel`, `FloatingChatPanel`, `MessageList`, `MessageBubble`, `ChatInput`, `ModelSelector`, `ThinkingIndicator`, `ToolCallDisplay`, `ToolResultDisplay`, `UsageSummary`
- React hooks for chat streams, persistence, memory, protocols, insights, and tool operations
- `AgentChatProvider` context and hook
- Full TypeScript type definitions for all API models
- CI workflow for automated npm publish on push to main
