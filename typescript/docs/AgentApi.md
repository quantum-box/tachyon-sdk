# AgentApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createAgentSession**](AgentApi.md#createagentsessionoperation) | **POST** /v1/llms/sessions | Create a new independent agent session. |
| [**executeAgent**](AgentApi.md#executeagent) | **POST** /v1/llms/chatrooms/{chatroom_id}/agent/execute | Execute agent |
| [**executeAgentSession**](AgentApi.md#executeagentsession) | **POST** /v1/llms/sessions/{session_id}/agent/execute | Execute agent via session endpoint. |
| [**getAgentMessages**](AgentApi.md#getagentmessages) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/messages | Get agent message log |
| [**getAgentMessagesSession**](AgentApi.md#getagentmessagessession) | **GET** /v1/llms/sessions/{session_id}/agent/messages | Get agent message log (session alias) |
| [**getAgentStatus**](AgentApi.md#getagentstatus) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/status | Get agent status |
| [**getAgentStatusSession**](AgentApi.md#getagentstatussession) | **GET** /v1/llms/sessions/{session_id}/agent/status | Get agent status (session alias) |
| [**listAgentSessions**](AgentApi.md#listagentsessions) | **GET** /v1/llms/sessions | List agent sessions for the current tenant. |
| [**submitToolResult**](AgentApi.md#submittoolresult) | **POST** /v1/llms/chatrooms/{chatroom_id}/agent/tool-result | Submit the result for a client-side tool call. |
| [**submitToolResultSession**](AgentApi.md#submittoolresultsession) | **POST** /v1/llms/sessions/{session_id}/agent/tool-result | Submit client tool result (session alias) |



## createAgentSession

> CreateAgentSessionOutputData createAgentSession(xOperatorId, authorization, createAgentSessionRequest)

Create a new independent agent session.

Returns a session with an &#x60;as_&#x60;-prefixed ID that can be used with the session agent endpoints.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { CreateAgentSessionOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // CreateAgentSessionRequest
    createAgentSessionRequest: ...,
  } satisfies CreateAgentSessionOperationRequest;

  try {
    const data = await api.createAgentSession(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **createAgentSessionRequest** | [CreateAgentSessionRequest](CreateAgentSessionRequest.md) |  | |

### Return type

[**CreateAgentSessionOutputData**](CreateAgentSessionOutputData.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Session created |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## executeAgent

> AgentChunk executeAgent(chatroomId, xOperatorId, authorization, agentExecuteRequest)

Execute agent

Execute a task using the agent system. Returns an SSE stream of &#x60;AgentChunk&#x60; events.  ## Client-Side Tools  You can provide custom tool definitions via the &#x60;client_tools&#x60; field. When the LLM decides to call one of these tools, the server emits a &#x60;tool_call_pending&#x60; SSE event containing the tool name, arguments, and a unique &#x60;tool_id&#x60;. The client should:  1. Execute the tool locally using the provided arguments 2. Submit the result via    &#x60;POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result&#x60;    with the matching &#x60;tool_id&#x60; 3. The agent resumes processing with the tool result  For **fire-and-forget** tools (&#x60;fire_and_forget: true&#x60;), the server immediately returns a success message to the LLM without waiting for a result submission. The client still receives the &#x60;tool_call_pending&#x60; event and can execute the tool, but does not need to submit a result.  Tool names must not collide with server built-in tools (e.g. &#x60;read_file&#x60;, &#x60;execute_command&#x60;). The server validates this and returns an error if a collision is detected.  When &#x60;client_tools&#x60; is provided, JSON tool call mode is automatically enabled — you do not need to set &#x60;use_json_tool_calls&#x60; separately.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { ExecuteAgentRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Chatroom ID
    chatroomId: chatroomId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // AgentExecuteRequest
    agentExecuteRequest: ...,
  } satisfies ExecuteAgentRequest;

  try {
    const data = await api.executeAgent(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **chatroomId** | `string` | Chatroom ID | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **agentExecuteRequest** | [AgentExecuteRequest](AgentExecuteRequest.md) |  | |

### Return type

[**AgentChunk**](AgentChunk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `text/event-stream`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | SSE stream of AgentChunk events. The stream ends with a &#x60;done&#x60; event. |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## executeAgentSession

> AgentChunk executeAgentSession(sessionId, xOperatorId, authorization, agentExecuteRequest)

Execute agent via session endpoint.

Accepts both independent session IDs (&#x60;as_...&#x60;) and legacy chatroom IDs (&#x60;ch_...&#x60;) for backwards compatibility.  ## Client-Side Tools  You can provide custom tool definitions via the &#x60;client_tools&#x60; field. When the LLM decides to call one of these tools, the server emits a &#x60;tool_call_pending&#x60; SSE event containing the tool name, arguments, and a unique &#x60;tool_id&#x60;. The client should:  1. Execute the tool locally using the provided arguments 2. Submit the result via    &#x60;POST /v1/llms/sessions/{session_id}/agent/tool-result&#x60;    with the matching &#x60;tool_id&#x60; 3. The agent resumes processing with the tool result  For **fire-and-forget** tools (&#x60;fire_and_forget: true&#x60;), the server immediately returns a success to the LLM without waiting for a result submission.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { ExecuteAgentSessionRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Session ID (`as_...`) or legacy chatroom ID (`ch_...`)
    sessionId: sessionId_example,
    // string | Operator ID
    xOperatorId: tn_01hjryxysgey07h5jz5wagqj0m,
    // string | Bearer token for authentication
    authorization: Bearer dummy-token,
    // AgentExecuteRequest
    agentExecuteRequest: {"auto_approve":true,"max_requests":5,"task":"Summarize the key points of the uploaded document."},
  } satisfies ExecuteAgentSessionRequest;

  try {
    const data = await api.executeAgentSession(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **sessionId** | `string` | Session ID (&#x60;as_...&#x60;) or legacy chatroom ID (&#x60;ch_...&#x60;) | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Bearer token for authentication | [Defaults to `undefined`] |
| **agentExecuteRequest** | [AgentExecuteRequest](AgentExecuteRequest.md) |  | |

### Return type

[**AgentChunk**](AgentChunk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `text/event-stream`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | SSE stream of AgentChunk events.  Each event is sent as &#x60;event: &lt;type&gt;\\ndata: &lt;json&gt;\\n\\n&#x60;. The stream ends with an &#x60;event: done&#x60; sentinel.  ## Event types | Event | Description | |-------|-------------| | &#x60;say&#x60; | Text content from the agent | | &#x60;thinking&#x60; | Reasoning/thinking content (may stream in chunks) | | &#x60;tool_call&#x60; | Agent initiates a tool call | | &#x60;tool_call_args&#x60; | Arguments for the tool call | | &#x60;tool_result&#x60; | Result of a tool execution | | &#x60;tool_call_pending&#x60; | Client-side tool awaiting result submission | | &#x60;tool_job_started&#x60; | External coding agent job started | | &#x60;ask&#x60; | Agent asks a follow-up question | | &#x60;attempt_completion&#x60; | Agent proposes task completion | | &#x60;user&#x60; | Echoed user message | | &#x60;usage&#x60; | Token usage statistics | | &#x60;error&#x60; | Error during execution | | &#x60;done&#x60; | Stream complete | |  -  |
| **401** | Unauthorized — missing or invalid Authorization header |  -  |
| **403** | Forbidden — operator does not have permission |  -  |
| **404** | Session not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getAgentMessages

> AgentMessagesResponse getAgentMessages(chatroomId, xOperatorId, authorization, limit, offset)

Get agent message log

Get agent message log for specified chatroom.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { GetAgentMessagesRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | チャットルームID
    chatroomId: chatroomId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // number | 取得するメッセージの最大数 (optional)
    limit: 56,
    // number | 取得開始位置 (optional)
    offset: 56,
  } satisfies GetAgentMessagesRequest;

  try {
    const data = await api.getAgentMessages(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **chatroomId** | `string` | チャットルームID | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **limit** | `number` | 取得するメッセージの最大数 | [Optional] [Defaults to `undefined`] |
| **offset** | `number` | 取得開始位置 | [Optional] [Defaults to `undefined`] |

### Return type

[**AgentMessagesResponse**](AgentMessagesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | メッセージログの取得に成功 |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getAgentMessagesSession

> AgentMessagesResponse getAgentMessagesSession(sessionId, xOperatorId, authorization, limit, offset)

Get agent message log (session alias)

Session-based alias of &#x60;GET /v1/llms/chatrooms/{chatroom_id}/agent/messages&#x60;.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { GetAgentMessagesSessionRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Session ID
    sessionId: sessionId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // number | Maximum number of messages (optional)
    limit: 56,
    // number | Offset (optional)
    offset: 56,
  } satisfies GetAgentMessagesSessionRequest;

  try {
    const data = await api.getAgentMessagesSession(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **sessionId** | `string` | Session ID | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **limit** | `number` | Maximum number of messages | [Optional] [Defaults to `undefined`] |
| **offset** | `number` | Offset | [Optional] [Defaults to `undefined`] |

### Return type

[**AgentMessagesResponse**](AgentMessagesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getAgentStatus

> AgentStatusResponse getAgentStatus(chatroomId, xOperatorId, authorization)

Get agent status

Get execution status of agent for specified chatroom.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { GetAgentStatusRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Chatroom ID
    chatroomId: chatroomId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
  } satisfies GetAgentStatusRequest;

  try {
    const data = await api.getAgentStatus(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **chatroomId** | `string` | Chatroom ID | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |

### Return type

[**AgentStatusResponse**](AgentStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | 成功レスポンス |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getAgentStatusSession

> AgentStatusResponse getAgentStatusSession(sessionId, xOperatorId, authorization)

Get agent status (session alias)

Session-based alias of &#x60;GET /v1/llms/chatrooms/{chatroom_id}/agent/status&#x60;.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { GetAgentStatusSessionRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Session ID
    sessionId: sessionId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
  } satisfies GetAgentStatusSessionRequest;

  try {
    const data = await api.getAgentStatusSession(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **sessionId** | `string` | Session ID | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |

### Return type

[**AgentStatusResponse**](AgentStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listAgentSessions

> ListAgentSessionsOutputData listAgentSessions(xOperatorId, authorization, limit, offset)

List agent sessions for the current tenant.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { ListAgentSessionsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // number | Maximum number of sessions (optional)
    limit: 56,
    // number | Offset (optional)
    offset: 56,
  } satisfies ListAgentSessionsRequest;

  try {
    const data = await api.listAgentSessions(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **limit** | `number` | Maximum number of sessions | [Optional] [Defaults to `undefined`] |
| **offset** | `number` | Offset | [Optional] [Defaults to `undefined`] |

### Return type

[**ListAgentSessionsOutputData**](ListAgentSessionsOutputData.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Session list |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## submitToolResult

> any submitToolResult(chatroomId, xOperatorId, authorization, toolResultSubmission)

Submit the result for a client-side tool call.

After receiving a &#x60;tool_call_pending&#x60; SSE event from the agent execute stream, call this endpoint with the matching &#x60;tool_id&#x60; and the tool execution result.  The agent is blocked waiting for this result (up to 5 minutes). Once submitted, the agent resumes processing with the provided output.  For **fire-and-forget** tools, this endpoint does not need to be called — the agent continues immediately. If called anyway, the server returns 404 (no pending tool call).  ## Flow  1. Client sends &#x60;POST /agent/execute&#x60; with &#x60;client_tools&#x60; 2. Server streams SSE events; emits &#x60;tool_call_pending&#x60;    when the LLM calls a client tool 3. Client executes the tool locally 4. Client calls this endpoint with the result 5. Server delivers the result to the agent and resumes

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { SubmitToolResultRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Chatroom ID where the agent is executing
    chatroomId: chatroomId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // ToolResultSubmission
    toolResultSubmission: {"is_error":false,"result":"Message sent successfully","tool_id":"ct_01JEXAMPLE"},
  } satisfies SubmitToolResultRequest;

  try {
    const data = await api.submitToolResult(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **chatroomId** | `string` | Chatroom ID where the agent is executing | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **toolResultSubmission** | [ToolResultSubmission](ToolResultSubmission.md) |  | |

### Return type

**any**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Result accepted and delivered to the agent |  -  |
| **404** | No pending tool call found for the given tool_id (already submitted, timed out, or fire-and-forget) |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## submitToolResultSession

> any submitToolResultSession(sessionId, xOperatorId, authorization, toolResultSubmission)

Submit client tool result (session alias)

Session-based alias of &#x60;POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result&#x60;.

### Example

```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { SubmitToolResultSessionRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Session ID where the agent is executing
    sessionId: sessionId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // ToolResultSubmission
    toolResultSubmission: ...,
  } satisfies SubmitToolResultSessionRequest;

  try {
    const data = await api.submitToolResultSession(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **sessionId** | `string` | Session ID where the agent is executing | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **toolResultSubmission** | [ToolResultSubmission](ToolResultSubmission.md) |  | |

### Return type

**any**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Result accepted and delivered to the agent |  -  |
| **404** | No pending tool call found for the given tool_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

