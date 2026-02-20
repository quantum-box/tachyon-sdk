# AgentChunkEventOneOf9

Usage statistics

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cache_creation_input_tokens** | **int** |  | [optional] 
**cache_read_input_tokens** | **int** |  | [optional] 
**completion_tokens** | **int** |  | 
**prompt_tokens** | **int** |  | 
**total_cost** | **float** |  | [optional] 
**total_tokens** | **int** |  | 
**type** | **str** |  | 

## Example

```python
from tachyon_sdk.models.agent_chunk_event_one_of9 import AgentChunkEventOneOf9

# TODO update the JSON string below
json = "{}"
# create an instance of AgentChunkEventOneOf9 from a JSON string
agent_chunk_event_one_of9_instance = AgentChunkEventOneOf9.from_json(json)
# print the JSON string representation of the object
print(AgentChunkEventOneOf9.to_json())

# convert the object into a dict
agent_chunk_event_one_of9_dict = agent_chunk_event_one_of9_instance.to_dict()
# create an instance of AgentChunkEventOneOf9 from a dict
agent_chunk_event_one_of9_from_dict = AgentChunkEventOneOf9.from_dict(agent_chunk_event_one_of9_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


