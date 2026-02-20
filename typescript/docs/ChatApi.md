# ChatApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**chatCompletion**](ChatApi.md#chatcompletionoperation) | **POST** /v1/llms/chat/completions | Create a chat completion |
| [**chatCompletionOnChatroom**](ChatApi.md#chatcompletiononchatroom) | **POST** /v1/llms/chatrooms/{chatroom_id}/chat/completions | Create a chat completion within a chatroom |



## chatCompletion

> ChatCompletionChunkResponse chatCompletion(xOperatorId, authorization, chatCompletionRequest)

Create a chat completion

Creates a model response for the given chat conversation.

### Example

```ts
import {
  Configuration,
  ChatApi,
} from '@tachyon/sdk';
import type { ChatCompletionOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new ChatApi();

  const body = {
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // ChatCompletionRequest
    chatCompletionRequest: ...,
  } satisfies ChatCompletionOperationRequest;

  try {
    const data = await api.chatCompletion(body);
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
| **chatCompletionRequest** | [ChatCompletionRequest](ChatCompletionRequest.md) |  | |

### Return type

[**ChatCompletionChunkResponse**](ChatCompletionChunkResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Streaming Successful response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## chatCompletionOnChatroom

> ChatCompletionWithChatroomStreamResponse chatCompletionOnChatroom(chatroomId, xOperatorId, authorization, chatCompletionRequest)

Create a chat completion within a chatroom

Creates a model response for the given chat conversation in a specific chatroom.

### Example

```ts
import {
  Configuration,
  ChatApi,
} from '@tachyon/sdk';
import type { ChatCompletionOnChatroomRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new ChatApi();

  const body = {
    // string | Chatroom ID
    chatroomId: chatroomId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // ChatCompletionRequest
    chatCompletionRequest: ...,
  } satisfies ChatCompletionOnChatroomRequest;

  try {
    const data = await api.chatCompletionOnChatroom(body);
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
| **chatCompletionRequest** | [ChatCompletionRequest](ChatCompletionRequest.md) |  | |

### Return type

[**ChatCompletionWithChatroomStreamResponse**](ChatCompletionWithChatroomStreamResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Successful response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

