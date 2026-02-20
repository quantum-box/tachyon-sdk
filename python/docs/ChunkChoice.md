# ChunkChoice


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**delta** | [**DeltaMessage**](DeltaMessage.md) | The delta message | 
**finish_reason** | **str** | The finish reason | [optional] 
**index** | **int** | The index of the choice | 

## Example

```python
from tachyon_sdk.models.chunk_choice import ChunkChoice

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkChoice from a JSON string
chunk_choice_instance = ChunkChoice.from_json(json)
# print the JSON string representation of the object
print(ChunkChoice.to_json())

# convert the object into a dict
chunk_choice_dict = chunk_choice_instance.to_dict()
# create an instance of ChunkChoice from a dict
chunk_choice_from_dict = ChunkChoice.from_dict(chunk_choice_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


