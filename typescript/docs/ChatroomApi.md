# ChatroomApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createChatroom**](ChatroomApi.md#createchatroom) | **POST** /v1/llms/chatrooms | Create a new chatroom |
| [**deleteChatroom**](ChatroomApi.md#deletechatroom) | **DELETE** /v1/llms/chatrooms/{chatroom_id} | Delete a chatroom |
| [**updateChatroom**](ChatroomApi.md#updatechatroomoperation) | **PATCH** /v1/llms/chatrooms/{chatroom_id} | Update a chatroom |



## createChatroom

> CreateChatRoomResponse createChatroom(xOperatorId, authorization, createChatRoomRequest)

Create a new chatroom

Creates a new chatroom for the authenticated user.

### Example

```ts
import {
  Configuration,
  ChatroomApi,
} from '@tachyon/sdk';
import type { CreateChatroomRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new ChatroomApi();

  const body = {
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // CreateChatRoomRequest
    createChatRoomRequest: ...,
  } satisfies CreateChatroomRequest;

  try {
    const data = await api.createChatroom(body);
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
| **createChatRoomRequest** | [CreateChatRoomRequest](CreateChatRoomRequest.md) |  | |

### Return type

[**CreateChatRoomResponse**](CreateChatRoomResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Chatroom created successfully |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deleteChatroom

> deleteChatroom(chatroomId, xOperatorId, authorization)

Delete a chatroom

### Example

```ts
import {
  Configuration,
  ChatroomApi,
} from '@tachyon/sdk';
import type { DeleteChatroomRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new ChatroomApi();

  const body = {
    // string | Chatroom ID
    chatroomId: chatroomId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
  } satisfies DeleteChatroomRequest;

  try {
    const data = await api.deleteChatroom(body);
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

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **204** | Chatroom deleted successfully |  -  |
| **400** | Invalid chatroom ID format |  -  |
| **404** | Chatroom not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateChatroom

> UpdateChatroomResponse updateChatroom(chatroomId, xOperatorId, authorization, updateChatroomRequest)

Update a chatroom

### Example

```ts
import {
  Configuration,
  ChatroomApi,
} from '@tachyon/sdk';
import type { UpdateChatroomOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new ChatroomApi();

  const body = {
    // string | Chatroom ID
    chatroomId: chatroomId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // UpdateChatroomRequest
    updateChatroomRequest: ...,
  } satisfies UpdateChatroomOperationRequest;

  try {
    const data = await api.updateChatroom(body);
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
| **updateChatroomRequest** | [UpdateChatroomRequest](UpdateChatroomRequest.md) |  | |

### Return type

[**UpdateChatroomResponse**](UpdateChatroomResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Chatroom updated successfully |  -  |
| **400** | Invalid chatroom ID or request |  -  |
| **404** | Chatroom not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

