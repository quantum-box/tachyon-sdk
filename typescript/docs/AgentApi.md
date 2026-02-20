# AgentApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**executeAgent**](AgentApi.md#executeagent) | **POST** /v1/llms/chatrooms/{chatroom_id}/agent/execute | Execute agent |
| [**getAgentMessages**](AgentApi.md#getagentmessages) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/messages | Get agent message log |
| [**getAgentStatus**](AgentApi.md#getagentstatus) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/status | Get agent status |



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

