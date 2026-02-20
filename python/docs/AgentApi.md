# tachyon_sdk.AgentApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**execute_agent**](AgentApi.md#execute_agent) | **POST** /v1/llms/chatrooms/{chatroom_id}/agent/execute | Execute agent
[**get_agent_messages**](AgentApi.md#get_agent_messages) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/messages | Get agent message log
[**get_agent_status**](AgentApi.md#get_agent_status) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/status | Get agent status


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

