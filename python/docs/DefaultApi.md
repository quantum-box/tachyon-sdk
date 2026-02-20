# tachyon_sdk.DefaultApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_chatroom_messages**](DefaultApi.md#get_chatroom_messages) | **GET** /v1/llms/chatrooms/{chatroom_id}/messages | Get messages from a chatroom
[**get_chatrooms**](DefaultApi.md#get_chatrooms) | **GET** /v1/llms/chatrooms | Get chatrooms list


# **get_chatroom_messages**
> ChatroomsChatroomIdMessagesGetResponse get_chatroom_messages(chatroom_id, limit, offset, search, x_operator_id, authorization)

Get messages from a chatroom

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.chatrooms_chatroom_id_messages_get_response import ChatroomsChatroomIdMessagesGetResponse
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
    api_instance = tachyon_sdk.DefaultApi(api_client)
    chatroom_id = 'chatroom_id_example' # str | Chatroom ID
    limit = 56 # int | Limit
    offset = 56 # int | Offset
    search = 'search_example' # str | Search
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization

    try:
        # Get messages from a chatroom
        api_response = api_instance.get_chatroom_messages(chatroom_id, limit, offset, search, x_operator_id, authorization)
        print("The response of DefaultApi->get_chatroom_messages:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DefaultApi->get_chatroom_messages: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **chatroom_id** | **str**| Chatroom ID | 
 **limit** | **int**| Limit | 
 **offset** | **int**| Offset | 
 **search** | **str**| Search | 
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 

### Return type

[**ChatroomsChatroomIdMessagesGetResponse**](ChatroomsChatroomIdMessagesGetResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Messages retrieved successfully |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_chatrooms**
> GetChatroomsResponse get_chatrooms(x_operator_id, authorization, limit=limit, offset=offset, search=search, metadata_user_id=metadata_user_id)

Get chatrooms list

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.get_chatrooms_response import GetChatroomsResponse
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
    api_instance = tachyon_sdk.DefaultApi(api_client)
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    limit = 56 # int | Maximum number of chatrooms to return (default: 20, max: 100) (optional)
    offset = 56 # int | Number of chatrooms to skip for pagination (optional)
    search = 'search_example' # str | Search chatrooms by name (partial match) (optional)
    metadata_user_id = 'metadata_user_id_example' # str | Filter chatrooms by `user_id` stored in metadata JSON. Use this to retrieve chatrooms associated with a specific external user (e.g., LINE user ID). Only chatrooms where `metadata.user_id` matches exactly will be returned. (optional)

    try:
        # Get chatrooms list
        api_response = api_instance.get_chatrooms(x_operator_id, authorization, limit=limit, offset=offset, search=search, metadata_user_id=metadata_user_id)
        print("The response of DefaultApi->get_chatrooms:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DefaultApi->get_chatrooms: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **limit** | **int**| Maximum number of chatrooms to return (default: 20, max: 100) | [optional] 
 **offset** | **int**| Number of chatrooms to skip for pagination | [optional] 
 **search** | **str**| Search chatrooms by name (partial match) | [optional] 
 **metadata_user_id** | **str**| Filter chatrooms by &#x60;user_id&#x60; stored in metadata JSON. Use this to retrieve chatrooms associated with a specific external user (e.g., LINE user ID). Only chatrooms where &#x60;metadata.user_id&#x60; matches exactly will be returned. | [optional] 

### Return type

[**GetChatroomsResponse**](GetChatroomsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Chatrooms retrieved successfully |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

