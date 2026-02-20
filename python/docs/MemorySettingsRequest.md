# MemorySettingsRequest

Memory settings for context building

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**extract_memory** | **bool** | Whether to extract memory from the message | [default to True]
**max_memories** | **int** | Maximum number of memories to retrieve | [default to 5]
**min_relevance_score** | **float** | Minimum relevance score for memory retrieval (0.0 to 1.0) | [default to 0.7]

## Example

```python
from tachyon_sdk.models.memory_settings_request import MemorySettingsRequest

# TODO update the JSON string below
json = "{}"
# create an instance of MemorySettingsRequest from a JSON string
memory_settings_request_instance = MemorySettingsRequest.from_json(json)
# print the JSON string representation of the object
print(MemorySettingsRequest.to_json())

# convert the object into a dict
memory_settings_request_dict = memory_settings_request_instance.to_dict()
# create an instance of MemorySettingsRequest from a dict
memory_settings_request_from_dict = MemorySettingsRequest.from_dict(memory_settings_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


