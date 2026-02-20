# ChatCompletionChunkResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**choices** | [**List[ChunkChoice]**](ChunkChoice.md) |  | 
**created** | **int** |  | 
**id** | **str** |  | 
**model** | **str** |  | 
**object** | **str** |  | 
**system_fingerprint** | **str** |  | [optional] 

## Example

```python
from tachyon_sdk.models.chat_completion_chunk_response import ChatCompletionChunkResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ChatCompletionChunkResponse from a JSON string
chat_completion_chunk_response_instance = ChatCompletionChunkResponse.from_json(json)
# print the JSON string representation of the object
print(ChatCompletionChunkResponse.to_json())

# convert the object into a dict
chat_completion_chunk_response_dict = chat_completion_chunk_response_instance.to_dict()
# create an instance of ChatCompletionChunkResponse from a dict
chat_completion_chunk_response_from_dict = ChatCompletionChunkResponse.from_dict(chat_completion_chunk_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


