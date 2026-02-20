# DefaultApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getChatroomMessages**](DefaultApi.md#getchatroommessages) | **GET** /v1/llms/chatrooms/{chatroom_id}/messages | Get messages from a chatroom |
| [**getChatrooms**](DefaultApi.md#getchatrooms) | **GET** /v1/llms/chatrooms | Get chatrooms list |



## getChatroomMessages

> ChatroomsChatroomIdMessagesGetResponse getChatroomMessages(chatroomId, limit, offset, search, xOperatorId, authorization)

Get messages from a chatroom

### Example

```ts
import {
  Configuration,
  DefaultApi,
} from '@tachyon/sdk';
import type { GetChatroomMessagesRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new DefaultApi();

  const body = {
    // string | Chatroom ID
    chatroomId: chatroomId_example,
    // number | Limit
    limit: 56,
    // number | Offset
    offset: 56,
    // string | Search
    search: search_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
  } satisfies GetChatroomMessagesRequest;

  try {
    const data = await api.getChatroomMessages(body);
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
| **limit** | `number` | Limit | [Defaults to `undefined`] |
| **offset** | `number` | Offset | [Defaults to `undefined`] |
| **search** | `string` | Search | [Defaults to `undefined`] |
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |

### Return type

[**ChatroomsChatroomIdMessagesGetResponse**](ChatroomsChatroomIdMessagesGetResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Messages retrieved successfully |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getChatrooms

> GetChatroomsResponse getChatrooms(xOperatorId, authorization, limit, offset, search, metadataUserId)

Get chatrooms list

### Example

```ts
import {
  Configuration,
  DefaultApi,
} from '@tachyon/sdk';
import type { GetChatroomsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new DefaultApi();

  const body = {
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // number | Maximum number of chatrooms to return (default: 20, max: 100) (optional)
    limit: 56,
    // number | Number of chatrooms to skip for pagination (optional)
    offset: 56,
    // string | Search chatrooms by name (partial match) (optional)
    search: search_example,
    // string | Filter chatrooms by `user_id` stored in metadata JSON. Use this to retrieve chatrooms associated with a specific external user (e.g., LINE user ID). Only chatrooms where `metadata.user_id` matches exactly will be returned. (optional)
    metadataUserId: metadataUserId_example,
  } satisfies GetChatroomsRequest;

  try {
    const data = await api.getChatrooms(body);
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
| **limit** | `number` | Maximum number of chatrooms to return (default: 20, max: 100) | [Optional] [Defaults to `undefined`] |
| **offset** | `number` | Number of chatrooms to skip for pagination | [Optional] [Defaults to `undefined`] |
| **search** | `string` | Search chatrooms by name (partial match) | [Optional] [Defaults to `undefined`] |
| **metadataUserId** | `string` | Filter chatrooms by &#x60;user_id&#x60; stored in metadata JSON. Use this to retrieve chatrooms associated with a specific external user (e.g., LINE user ID). Only chatrooms where &#x60;metadata.user_id&#x60; matches exactly will be returned. | [Optional] [Defaults to `undefined`] |

### Return type

[**GetChatroomsResponse**](GetChatroomsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Chatrooms retrieved successfully |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

