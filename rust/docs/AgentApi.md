# \AgentApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_agent_session**](AgentApi.md#create_agent_session) | **POST** /v1/llms/sessions | Create a new independent agent session.
[**execute_agent**](AgentApi.md#execute_agent) | **POST** /v1/llms/chatrooms/{chatroom_id}/agent/execute | Execute agent
[**execute_agent_session**](AgentApi.md#execute_agent_session) | **POST** /v1/llms/sessions/{session_id}/agent/execute | Execute agent via session endpoint.
[**get_agent_messages**](AgentApi.md#get_agent_messages) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/messages | Get agent message log
[**get_agent_messages_session**](AgentApi.md#get_agent_messages_session) | **GET** /v1/llms/sessions/{session_id}/agent/messages | Get agent message log (session alias)
[**get_agent_status**](AgentApi.md#get_agent_status) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/status | Get agent status
[**get_agent_status_session**](AgentApi.md#get_agent_status_session) | **GET** /v1/llms/sessions/{session_id}/agent/status | Get agent status (session alias)
[**list_agent_sessions**](AgentApi.md#list_agent_sessions) | **GET** /v1/llms/sessions | List agent sessions for the current tenant.
[**submit_tool_result**](AgentApi.md#submit_tool_result) | **POST** /v1/llms/chatrooms/{chatroom_id}/agent/tool-result | Submit the result for a client-side tool call.
[**submit_tool_result_session**](AgentApi.md#submit_tool_result_session) | **POST** /v1/llms/sessions/{session_id}/agent/tool-result | Submit client tool result (session alias)



## create_agent_session

> models::CreateAgentSessionOutputData create_agent_session(x_operator_id, authorization, create_agent_session_request)
Create a new independent agent session.

Returns a session with an `as_`-prefixed ID that can be used with the session agent endpoints.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |
**create_agent_session_request** | [**CreateAgentSessionRequest**](CreateAgentSessionRequest.md) |  | [required] |

### Return type

[**models::CreateAgentSessionOutputData**](CreateAgentSessionOutputData.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## execute_agent

> models::AgentChunk execute_agent(chatroom_id, x_operator_id, authorization, agent_execute_request)
Execute agent

Execute a task using the agent system. Returns an SSE stream of `AgentChunk` events.  ## Client-Side Tools  You can provide custom tool definitions via the `client_tools` field. When the LLM decides to call one of these tools, the server emits a `tool_call_pending` SSE event containing the tool name, arguments, and a unique `tool_id`. The client should:  1. Execute the tool locally using the provided arguments 2. Submit the result via    `POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result`    with the matching `tool_id` 3. The agent resumes processing with the tool result  For **fire-and-forget** tools (`fire_and_forget: true`), the server immediately returns a success message to the LLM without waiting for a result submission. The client still receives the `tool_call_pending` event and can execute the tool, but does not need to submit a result.  Tool names must not collide with server built-in tools (e.g. `read_file`, `execute_command`). The server validates this and returns an error if a collision is detected.  When `client_tools` is provided, JSON tool call mode is automatically enabled — you do not need to set `use_json_tool_calls` separately.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**chatroom_id** | **String** | Chatroom ID | [required] |
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |
**agent_execute_request** | [**AgentExecuteRequest**](AgentExecuteRequest.md) |  | [required] |

### Return type

[**models::AgentChunk**](AgentChunk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: text/event-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## execute_agent_session

> models::AgentChunk execute_agent_session(session_id, x_operator_id, authorization, agent_execute_request)
Execute agent via session endpoint.

Accepts both independent session IDs (`as_...`) and legacy chatroom IDs (`ch_...`) for backwards compatibility.  ## Client-Side Tools  You can provide custom tool definitions via the `client_tools` field. When the LLM decides to call one of these tools, the server emits a `tool_call_pending` SSE event containing the tool name, arguments, and a unique `tool_id`. The client should:  1. Execute the tool locally using the provided arguments 2. Submit the result via    `POST /v1/llms/sessions/{session_id}/agent/tool-result`    with the matching `tool_id` 3. The agent resumes processing with the tool result  For **fire-and-forget** tools (`fire_and_forget: true`), the server immediately returns a success to the LLM without waiting for a result submission.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID (`as_...`) or legacy chatroom ID (`ch_...`) | [required] |
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Bearer token for authentication | [required] |
**agent_execute_request** | [**AgentExecuteRequest**](AgentExecuteRequest.md) |  | [required] |

### Return type

[**models::AgentChunk**](AgentChunk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: text/event-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_agent_messages

> models::AgentMessagesResponse get_agent_messages(chatroom_id, x_operator_id, authorization, limit, offset)
Get agent message log

Get agent message log for specified chatroom.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**chatroom_id** | **String** | チャットルームID | [required] |
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |
**limit** | Option<**i32**> | 取得するメッセージの最大数 |  |
**offset** | Option<**i32**> | 取得開始位置 |  |

### Return type

[**models::AgentMessagesResponse**](AgentMessagesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_agent_messages_session

> models::AgentMessagesResponse get_agent_messages_session(session_id, x_operator_id, authorization, limit, offset)
Get agent message log (session alias)

Session-based alias of `GET /v1/llms/chatrooms/{chatroom_id}/agent/messages`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |
**limit** | Option<**i32**> | Maximum number of messages |  |
**offset** | Option<**i32**> | Offset |  |

### Return type

[**models::AgentMessagesResponse**](AgentMessagesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_agent_status

> models::AgentStatusResponse get_agent_status(chatroom_id, x_operator_id, authorization)
Get agent status

Get execution status of agent for specified chatroom.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**chatroom_id** | **String** | Chatroom ID | [required] |
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |

### Return type

[**models::AgentStatusResponse**](AgentStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_agent_status_session

> models::AgentStatusResponse get_agent_status_session(session_id, x_operator_id, authorization)
Get agent status (session alias)

Session-based alias of `GET /v1/llms/chatrooms/{chatroom_id}/agent/status`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |

### Return type

[**models::AgentStatusResponse**](AgentStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_agent_sessions

> models::ListAgentSessionsOutputData list_agent_sessions(x_operator_id, authorization, limit, offset)
List agent sessions for the current tenant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |
**limit** | Option<**i32**> | Maximum number of sessions |  |
**offset** | Option<**i32**> | Offset |  |

### Return type

[**models::ListAgentSessionsOutputData**](ListAgentSessionsOutputData.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## submit_tool_result

> serde_json::Value submit_tool_result(chatroom_id, x_operator_id, authorization, tool_result_submission)
Submit the result for a client-side tool call.

After receiving a `tool_call_pending` SSE event from the agent execute stream, call this endpoint with the matching `tool_id` and the tool execution result.  The agent is blocked waiting for this result (up to 5 minutes). Once submitted, the agent resumes processing with the provided output.  For **fire-and-forget** tools, this endpoint does not need to be called — the agent continues immediately. If called anyway, the server returns 404 (no pending tool call).  ## Flow  1. Client sends `POST /agent/execute` with `client_tools` 2. Server streams SSE events; emits `tool_call_pending`    when the LLM calls a client tool 3. Client executes the tool locally 4. Client calls this endpoint with the result 5. Server delivers the result to the agent and resumes

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**chatroom_id** | **String** | Chatroom ID where the agent is executing | [required] |
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |
**tool_result_submission** | [**ToolResultSubmission**](ToolResultSubmission.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## submit_tool_result_session

> serde_json::Value submit_tool_result_session(session_id, x_operator_id, authorization, tool_result_submission)
Submit client tool result (session alias)

Session-based alias of `POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID where the agent is executing | [required] |
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Authorization | [required] |
**tool_result_submission** | [**ToolResultSubmission**](ToolResultSubmission.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

