# tachyon_sdk.ChatroomApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_chatroom**](ChatroomApi.md#create_chatroom) | **POST** /v1/llms/chatrooms | Create a new chatroom
[**delete_chatroom**](ChatroomApi.md#delete_chatroom) | **DELETE** /v1/llms/chatrooms/{chatroom_id} | Delete a chatroom
[**update_chatroom**](ChatroomApi.md#update_chatroom) | **PATCH** /v1/llms/chatrooms/{chatroom_id} | Update a chatroom


# **create_chatroom**
> CreateChatRoomResponse create_chatroom(x_operator_id, authorization, create_chat_room_request)

Create a new chatroom

Creates a new chatroom for the authenticated user.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_chat_room_request import CreateChatRoomRequest
from tachyon_sdk.models.create_chat_room_response import CreateChatRoomResponse
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
    api_instance = tachyon_sdk.ChatroomApi(api_client)
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    create_chat_room_request = tachyon_sdk.CreateChatRoomRequest() # CreateChatRoomRequest | 

    try:
        # Create a new chatroom
        api_response = api_instance.create_chatroom(x_operator_id, authorization, create_chat_room_request)
        print("The response of ChatroomApi->create_chatroom:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChatroomApi->create_chatroom: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **create_chat_room_request** | [**CreateChatRoomRequest**](CreateChatRoomRequest.md)|  | 

### Return type

[**CreateChatRoomResponse**](CreateChatRoomResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chatroom created successfully |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_chatroom**
> delete_chatroom(chatroom_id, x_operator_id, authorization)

Delete a chatroom

### Example


```python
import tachyon_sdk
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
    api_instance = tachyon_sdk.ChatroomApi(api_client)
    chatroom_id = 'chatroom_id_example' # str | Chatroom ID
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization

    try:
        # Delete a chatroom
        api_instance.delete_chatroom(chatroom_id, x_operator_id, authorization)
    except Exception as e:
        print("Exception when calling ChatroomApi->delete_chatroom: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chatroom_id** | **str**| Chatroom ID | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Chatroom deleted successfully |  -  |
**400** | Invalid chatroom ID format |  -  |
**404** | Chatroom not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_chatroom**
> UpdateChatroomResponse update_chatroom(chatroom_id, x_operator_id, authorization, update_chatroom_request)

Update a chatroom

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.update_chatroom_request import UpdateChatroomRequest
from tachyon_sdk.models.update_chatroom_response import UpdateChatroomResponse
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
    api_instance = tachyon_sdk.ChatroomApi(api_client)
    chatroom_id = 'chatroom_id_example' # str | Chatroom ID
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    update_chatroom_request = tachyon_sdk.UpdateChatroomRequest() # UpdateChatroomRequest | 

    try:
        # Update a chatroom
        api_response = api_instance.update_chatroom(chatroom_id, x_operator_id, authorization, update_chatroom_request)
        print("The response of ChatroomApi->update_chatroom:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ChatroomApi->update_chatroom: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chatroom_id** | **str**| Chatroom ID | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **update_chatroom_request** | [**UpdateChatroomRequest**](UpdateChatroomRequest.md)|  | 

### Return type

[**UpdateChatroomResponse**](UpdateChatroomResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chatroom updated successfully |  -  |
**400** | Invalid chatroom ID or request |  -  |
**404** | Chatroom not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

