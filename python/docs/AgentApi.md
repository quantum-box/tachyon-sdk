# tachyon_sdk.AgentApi

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


# **create_agent_session**
> CreateAgentSessionOutputData create_agent_session(x_operator_id, authorization, create_agent_session_request)

Create a new independent agent session.

Returns a session with an `as_`-prefixed ID that can be
used with the session agent endpoints.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_agent_session_output_data import CreateAgentSessionOutputData
from tachyon_sdk.models.create_agent_session_request import CreateAgentSessionRequest
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    create_agent_session_request = tachyon_sdk.CreateAgentSessionRequest() # CreateAgentSessionRequest | 

    try:
        # Create a new independent agent session.
        api_response = api_instance.create_agent_session(x_operator_id, authorization, create_agent_session_request)
        print("The response of AgentApi->create_agent_session:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->create_agent_session: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **create_agent_session_request** | [**CreateAgentSessionRequest**](CreateAgentSessionRequest.md)|  | 

### Return type

[**CreateAgentSessionOutputData**](CreateAgentSessionOutputData.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Session created |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **execute_agent**
> AgentChunk execute_agent(chatroom_id, x_operator_id, authorization, agent_execute_request)

Execute agent

Execute a task using the agent system. Returns an SSE
stream of `AgentChunk` events.

## Client-Side Tools

You can provide custom tool definitions via the
`client_tools` field. When the LLM decides to call one
of these tools, the server emits a `tool_call_pending`
SSE event containing the tool name, arguments, and a
unique `tool_id`. The client should:

1. Execute the tool locally using the provided arguments
2. Submit the result via
   `POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result`
   with the matching `tool_id`
3. The agent resumes processing with the tool result

For **fire-and-forget** tools (`fire_and_forget: true`),
the server immediately returns a success message to the
LLM without waiting for a result submission. The client
still receives the `tool_call_pending` event and can
execute the tool, but does not need to submit a result.

Tool names must not collide with server built-in tools
(e.g. `read_file`, `execute_command`). The server
validates this and returns an error if a collision is
detected.

When `client_tools` is provided, JSON tool call mode
is automatically enabled — you do not need to set
`use_json_tool_calls` separately.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.agent_chunk import AgentChunk
from tachyon_sdk.models.agent_execute_request import AgentExecuteRequest
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    chatroom_id = 'chatroom_id_example' # str | Chatroom ID
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    agent_execute_request = tachyon_sdk.AgentExecuteRequest() # AgentExecuteRequest | 

    try:
        # Execute agent
        api_response = api_instance.execute_agent(chatroom_id, x_operator_id, authorization, agent_execute_request)
        print("The response of AgentApi->execute_agent:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->execute_agent: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chatroom_id** | **str**| Chatroom ID | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **agent_execute_request** | [**AgentExecuteRequest**](AgentExecuteRequest.md)|  | 

### Return type

[**AgentChunk**](AgentChunk.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: text/event-stream

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | SSE stream of AgentChunk events. The stream ends with a &#x60;done&#x60; event. |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **execute_agent_session**
> AgentChunk execute_agent_session(session_id, x_operator_id, authorization, agent_execute_request)

Execute agent via session endpoint.

Accepts both independent session IDs (`as_...`) and legacy
chatroom IDs (`ch_...`) for backwards compatibility.

## Client-Side Tools

You can provide custom tool definitions via the
`client_tools` field. When the LLM decides to call one
of these tools, the server emits a `tool_call_pending`
SSE event containing the tool name, arguments, and a
unique `tool_id`. The client should:

1. Execute the tool locally using the provided arguments
2. Submit the result via
   `POST /v1/llms/sessions/{session_id}/agent/tool-result`
   with the matching `tool_id`
3. The agent resumes processing with the tool result

For **fire-and-forget** tools (`fire_and_forget: true`),
the server immediately returns a success to the LLM
without waiting for a result submission.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.agent_chunk import AgentChunk
from tachyon_sdk.models.agent_execute_request import AgentExecuteRequest
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    session_id = 'session_id_example' # str | Session ID (`as_...`) or legacy chatroom ID (`ch_...`)
    x_operator_id = 'tn_01hjryxysgey07h5jz5wagqj0m' # str | Operator ID
    authorization = 'Bearer dummy-token' # str | Bearer token for authentication
    agent_execute_request = {"auto_approve":true,"max_requests":5,"task":"Summarize the key points of the uploaded document."} # AgentExecuteRequest | 

    try:
        # Execute agent via session endpoint.
        api_response = api_instance.execute_agent_session(session_id, x_operator_id, authorization, agent_execute_request)
        print("The response of AgentApi->execute_agent_session:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->execute_agent_session: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **session_id** | **str**| Session ID (&#x60;as_...&#x60;) or legacy chatroom ID (&#x60;ch_...&#x60;) | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Bearer token for authentication | 
 **agent_execute_request** | [**AgentExecuteRequest**](AgentExecuteRequest.md)|  | 

### Return type

[**AgentChunk**](AgentChunk.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: text/event-stream

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | SSE stream of AgentChunk events.  Each event is sent as &#x60;event: &lt;type&gt;\\ndata: &lt;json&gt;\\n\\n&#x60;. The stream ends with an &#x60;event: done&#x60; sentinel.  ## Event types | Event | Description | |-------|-------------| | &#x60;say&#x60; | Text content from the agent | | &#x60;thinking&#x60; | Reasoning/thinking content (may stream in chunks) | | &#x60;tool_call&#x60; | Agent initiates a tool call | | &#x60;tool_call_args&#x60; | Arguments for the tool call | | &#x60;tool_result&#x60; | Result of a tool execution | | &#x60;tool_call_pending&#x60; | Client-side tool awaiting result submission | | &#x60;tool_job_started&#x60; | External coding agent job started | | &#x60;ask&#x60; | Agent asks a follow-up question | | &#x60;attempt_completion&#x60; | Agent proposes task completion | | &#x60;user&#x60; | Echoed user message | | &#x60;usage&#x60; | Token usage statistics | | &#x60;error&#x60; | Error during execution | | &#x60;done&#x60; | Stream complete | |  -  |
**401** | Unauthorized — missing or invalid Authorization header |  -  |
**403** | Forbidden — operator does not have permission |  -  |
**404** | Session not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_agent_messages**
> AgentMessagesResponse get_agent_messages(chatroom_id, x_operator_id, authorization, limit=limit, offset=offset)

Get agent message log

Get agent message log for specified chatroom.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.agent_messages_response import AgentMessagesResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    chatroom_id = 'chatroom_id_example' # str | チャットルームID
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    limit = 56 # int | 取得するメッセージの最大数 (optional)
    offset = 56 # int | 取得開始位置 (optional)

    try:
        # Get agent message log
        api_response = api_instance.get_agent_messages(chatroom_id, x_operator_id, authorization, limit=limit, offset=offset)
        print("The response of AgentApi->get_agent_messages:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->get_agent_messages: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chatroom_id** | **str**| チャットルームID | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **limit** | **int**| 取得するメッセージの最大数 | [optional] 
 **offset** | **int**| 取得開始位置 | [optional] 

### Return type

[**AgentMessagesResponse**](AgentMessagesResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | メッセージログの取得に成功 |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_agent_messages_session**
> AgentMessagesResponse get_agent_messages_session(session_id, x_operator_id, authorization, limit=limit, offset=offset)

Get agent message log (session alias)

Session-based alias of `GET /v1/llms/chatrooms/{chatroom_id}/agent/messages`.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.agent_messages_response import AgentMessagesResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    session_id = 'session_id_example' # str | Session ID
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    limit = 56 # int | Maximum number of messages (optional)
    offset = 56 # int | Offset (optional)

    try:
        # Get agent message log (session alias)
        api_response = api_instance.get_agent_messages_session(session_id, x_operator_id, authorization, limit=limit, offset=offset)
        print("The response of AgentApi->get_agent_messages_session:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->get_agent_messages_session: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **session_id** | **str**| Session ID | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **limit** | **int**| Maximum number of messages | [optional] 
 **offset** | **int**| Offset | [optional] 

### Return type

[**AgentMessagesResponse**](AgentMessagesResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Success response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_agent_status**
> AgentStatusResponse get_agent_status(chatroom_id, x_operator_id, authorization)

Get agent status

Get execution status of agent for specified chatroom.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.agent_status_response import AgentStatusResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    chatroom_id = 'chatroom_id_example' # str | Chatroom ID
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization

    try:
        # Get agent status
        api_response = api_instance.get_agent_status(chatroom_id, x_operator_id, authorization)
        print("The response of AgentApi->get_agent_status:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->get_agent_status: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chatroom_id** | **str**| Chatroom ID | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 

### Return type

[**AgentStatusResponse**](AgentStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | 成功レスポンス |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_agent_status_session**
> AgentStatusResponse get_agent_status_session(session_id, x_operator_id, authorization)

Get agent status (session alias)

Session-based alias of `GET /v1/llms/chatrooms/{chatroom_id}/agent/status`.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.agent_status_response import AgentStatusResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    session_id = 'session_id_example' # str | Session ID
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization

    try:
        # Get agent status (session alias)
        api_response = api_instance.get_agent_status_session(session_id, x_operator_id, authorization)
        print("The response of AgentApi->get_agent_status_session:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->get_agent_status_session: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **session_id** | **str**| Session ID | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 

### Return type

[**AgentStatusResponse**](AgentStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Success response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_agent_sessions**
> ListAgentSessionsOutputData list_agent_sessions(x_operator_id, authorization, limit=limit, offset=offset)

List agent sessions for the current tenant.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.list_agent_sessions_output_data import ListAgentSessionsOutputData
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    limit = 56 # int | Maximum number of sessions (optional)
    offset = 56 # int | Offset (optional)

    try:
        # List agent sessions for the current tenant.
        api_response = api_instance.list_agent_sessions(x_operator_id, authorization, limit=limit, offset=offset)
        print("The response of AgentApi->list_agent_sessions:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->list_agent_sessions: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **limit** | **int**| Maximum number of sessions | [optional] 
 **offset** | **int**| Offset | [optional] 

### Return type

[**ListAgentSessionsOutputData**](ListAgentSessionsOutputData.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Session list |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **submit_tool_result**
> object submit_tool_result(chatroom_id, x_operator_id, authorization, tool_result_submission)

Submit the result for a client-side tool call.

After receiving a `tool_call_pending` SSE event from the
agent execute stream, call this endpoint with the
matching `tool_id` and the tool execution result.

The agent is blocked waiting for this result (up to 5
minutes). Once submitted, the agent resumes processing
with the provided output.

For **fire-and-forget** tools, this endpoint does not
need to be called — the agent continues immediately.
If called anyway, the server returns 404 (no pending
tool call).

## Flow

1. Client sends `POST /agent/execute` with `client_tools`
2. Server streams SSE events; emits `tool_call_pending`
   when the LLM calls a client tool
3. Client executes the tool locally
4. Client calls this endpoint with the result
5. Server delivers the result to the agent and resumes

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.tool_result_submission import ToolResultSubmission
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    chatroom_id = 'chatroom_id_example' # str | Chatroom ID where the agent is executing
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    tool_result_submission = {"is_error":false,"result":"Message sent successfully","tool_id":"ct_01JEXAMPLE"} # ToolResultSubmission | 

    try:
        # Submit the result for a client-side tool call.
        api_response = api_instance.submit_tool_result(chatroom_id, x_operator_id, authorization, tool_result_submission)
        print("The response of AgentApi->submit_tool_result:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->submit_tool_result: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chatroom_id** | **str**| Chatroom ID where the agent is executing | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **tool_result_submission** | [**ToolResultSubmission**](ToolResultSubmission.md)|  | 

### Return type

**object**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Result accepted and delivered to the agent |  -  |
**404** | No pending tool call found for the given tool_id (already submitted, timed out, or fire-and-forget) |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **submit_tool_result_session**
> object submit_tool_result_session(session_id, x_operator_id, authorization, tool_result_submission)

Submit client tool result (session alias)

Session-based alias of `POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result`.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.tool_result_submission import ToolResultSubmission
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AgentApi(api_client)
    session_id = 'session_id_example' # str | Session ID where the agent is executing
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    tool_result_submission = tachyon_sdk.ToolResultSubmission() # ToolResultSubmission | 

    try:
        # Submit client tool result (session alias)
        api_response = api_instance.submit_tool_result_session(session_id, x_operator_id, authorization, tool_result_submission)
        print("The response of AgentApi->submit_tool_result_session:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AgentApi->submit_tool_result_session: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **session_id** | **str**| Session ID where the agent is executing | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **tool_result_submission** | [**ToolResultSubmission**](ToolResultSubmission.md)|  | 

### Return type

**object**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Result accepted and delivered to the agent |  -  |
**404** | No pending tool call found for the given tool_id |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

