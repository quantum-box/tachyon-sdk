# AgentChunkEventOneOf2

Tool execution result

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**is_finished** | **bool** |  | 
**result** | **str** |  | 
**tool_id** | **str** |  | 
**type** | **str** |  | 

## Example

```python
from tachyon_sdk.models.agent_chunk_event_one_of2 import AgentChunkEventOneOf2

# TODO update the JSON string below
json = "{}"
# create an instance of AgentChunkEventOneOf2 from a JSON string
agent_chunk_event_one_of2_instance = AgentChunkEventOneOf2.from_json(json)
# print the JSON string representation of the object
print(AgentChunkEventOneOf2.to_json())

# convert the object into a dict
agent_chunk_event_one_of2_dict = agent_chunk_event_one_of2_instance.to_dict()
# create an instance of AgentChunkEventOneOf2 from a dict
agent_chunk_event_one_of2_from_dict = AgentChunkEventOneOf2.from_dict(agent_chunk_event_one_of2_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


