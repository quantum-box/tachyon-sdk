# AgentExecuteRequest

TODO: add English documentation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**additional_tool_description** | **str** | Additional description appended to each tool&#39;s help text shown to the LLM. | [optional] 
**agent_protocol_id** | **str** | Optional AgentProtocol ID to use for this execution. | [optional] 
**agent_protocol_mode** | [**AgentProtocolMode**](AgentProtocolMode.md) | AgentProtocol selection mode. | [optional] 
**assistant_name** | **str** | Display name for the assistant in the conversation. | [optional] 
**auto_approve** | **bool** | When true, the agent auto-approves tool executions without asking for confirmation. | [optional] 
**chatroom_name_generation** | [**ChatroomNameGeneration**](ChatroomNameGeneration.md) | Controls chatroom name auto-generation behavior. - &#x60;first_only&#x60; (default): Generate on first message only (regardless of current name) - &#x60;always&#x60;: Always attempt to generate/update name after each message - &#x60;never&#x60;: Never auto-generate chatroom name | [optional] 
**client_tools** | [**List[ClientToolDefinition]**](ClientToolDefinition.md) | User-defined tools executed on the client side.  Each tool must have a unique &#x60;name&#x60; that does not collide with server built-in tool names. When the LLM calls one of these tools, the server emits a &#x60;tool_call_pending&#x60; SSE event and blocks until the client submits the result via &#x60;POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result&#x60; (timeout: 5 minutes).  Tools with &#x60;fire_and_forget: true&#x60; do not block — the server immediately returns a success to the LLM and the client can execute the tool asynchronously.  Example: &#x60;&#x60;&#x60;json {   \&quot;client_tools\&quot;: [     {       \&quot;name\&quot;: \&quot;send_slack_message\&quot;,       \&quot;description\&quot;: \&quot;Send a message to a Slack channel\&quot;,       \&quot;parameters\&quot;: {         \&quot;type\&quot;: \&quot;object\&quot;,         \&quot;properties\&quot;: {           \&quot;channel\&quot;: { \&quot;type\&quot;: \&quot;string\&quot; },           \&quot;text\&quot;: { \&quot;type\&quot;: \&quot;string\&quot; }         },         \&quot;required\&quot;: [\&quot;channel\&quot;, \&quot;text\&quot;]       },       \&quot;fire_and_forget\&quot;: false     }   ] } &#x60;&#x60;&#x60; | [optional] 
**max_requests** | **int** | Maximum number of LLM round-trips (tool calls) the agent is allowed to make before stopping. | [optional] 
**mcp_hub_config_json** | **str** | Optional MCP Hub configuration in JSON string form. | [optional] 
**model** | **str** | Model identifier. Accepts &#x60;provider/model&#x60; or just &#x60;model&#x60; (provider auto-detected).  Examples: - &#x60;anthropic/claude-3-sonnet-20241022&#x60; - &#x60;openai/gpt-4&#x60; - &#x60;google_ai/gemini-pro&#x60;  Auto-detection shortcuts: - &#x60;gpt-*&#x60; → OpenAI - &#x60;claude-*&#x60; → Anthropic - &#x60;gemini*&#x60; → Google AI | [optional] 
**task** | **str** | The task or prompt for the agent to execute. | 
**tool_access** | [**AgentToolAccessRequest**](AgentToolAccessRequest.md) | Per-category tool access flags. | [optional] 
**use_json_tool_calls** | **bool** | When true, use JSON Schema tool definitions (function calling) instead of XML-based tool parsing. This is automatically enabled when &#x60;client_tools&#x60; is provided. | [optional] 
**user_custom_instructions** | **str** | Custom system-level instructions appended to the agent&#39;s prompt. | [optional] 

## Example

```python
from tachyon_sdk.models.agent_execute_request import AgentExecuteRequest

# TODO update the JSON string below
json = "{}"
# create an instance of AgentExecuteRequest from a JSON string
agent_execute_request_instance = AgentExecuteRequest.from_json(json)
# print the JSON string representation of the object
print(AgentExecuteRequest.to_json())

# convert the object into a dict
agent_execute_request_dict = agent_execute_request_instance.to_dict()
# create an instance of AgentExecuteRequest from a dict
agent_execute_request_from_dict = AgentExecuteRequest.from_dict(agent_execute_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


