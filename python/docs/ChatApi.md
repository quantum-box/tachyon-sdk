# tachyon_sdk.ChatApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**chat_completion**](ChatApi.md#chat_completion) | **POST** /v1/llms/chat/completions | Create a chat completion
[**chat_completion_on_chatroom**](ChatApi.md#chat_completion_on_chatroom) | **POST** /v1/llms/chatrooms/{chatroom_id}/chat/completions | Create a chat completion within a chatroom


# **chat_completion**
> ChatCompletionChunkResponse chat_completion(x_operator_id, authorization, chat_completion_request)

Create a chat completion

Creates a model response for the given chat conversation.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.chat_completion_chunk_response import ChatCompletionChunkResponse
from tachyon_sdk.models.chat_completion_request import ChatCompletionRequest
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
    api_instance = tachyon_sdk.ChatApi(api_client)
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    chat_completion_request = tachyon_sdk.ChatCompletionRequest() # ChatCompletionRequest | 

    try:
        # Create a chat completion
        api_response = api_instance.chat_completion(x_operator_id, authorization, chat_completion_request)
        print("The response of ChatApi->chat_completion:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChatApi->chat_completion: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **chat_completion_request** | [**ChatCompletionRequest**](ChatCompletionRequest.md)|  | 

### Return type

[**ChatCompletionChunkResponse**](ChatCompletionChunkResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Streaming Successful response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **chat_completion_on_chatroom**
> ChatCompletionWithChatroomStreamResponse chat_completion_on_chatroom(chatroom_id, x_operator_id, authorization, chat_completion_request)

Create a chat completion within a chatroom

Creates a model response for the given chat conversation in a specific chatroom.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.chat_completion_request import ChatCompletionRequest
from tachyon_sdk.models.chat_completion_with_chatroom_stream_response import ChatCompletionWithChatroomStreamResponse
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
    api_instance = tachyon_sdk.ChatApi(api_client)
    chatroom_id = 'chatroom_id_example' # str | Chatroom ID
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    chat_completion_request = tachyon_sdk.ChatCompletionRequest() # ChatCompletionRequest | 

    try:
        # Create a chat completion within a chatroom
        api_response = api_instance.chat_completion_on_chatroom(chatroom_id, x_operator_id, authorization, chat_completion_request)
        print("The response of ChatApi->chat_completion_on_chatroom:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChatApi->chat_completion_on_chatroom: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chatroom_id** | **str**| Chatroom ID | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **chat_completion_request** | [**ChatCompletionRequest**](ChatCompletionRequest.md)|  | 

### Return type

[**ChatCompletionWithChatroomStreamResponse**](ChatCompletionWithChatroomStreamResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Successful response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

