# ChatCompletionWithChatroomStreamResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**choices** | [**List[ChunkChoice]**](ChunkChoice.md) | The choices generated | 
**created** | **int** | The created timestamp | [optional] 
**id** | **str** | The ID of the response | 
**model** | **str** | The model used | 
**object** | **str** | The object type | 

## Example

```python
from tachyon_sdk.models.chat_completion_with_chatroom_stream_response import ChatCompletionWithChatroomStreamResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ChatCompletionWithChatroomStreamResponse from a JSON string
chat_completion_with_chatroom_stream_response_instance = ChatCompletionWithChatroomStreamResponse.from_json(json)
# print the JSON string representation of the object
print(ChatCompletionWithChatroomStreamResponse.to_json())

# convert the object into a dict
chat_completion_with_chatroom_stream_response_dict = chat_completion_with_chatroom_stream_response_instance.to_dict()
# create an instance of ChatCompletionWithChatroomStreamResponse from a dict
chat_completion_with_chatroom_stream_response_from_dict = ChatCompletionWithChatroomStreamResponse.from_dict(chat_completion_with_chatroom_stream_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


