# AgentChunkEventOneOf

Tool call initiation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tool_id** | **str** |  | 
**tool_name** | **str** |  | 
**type** | **str** |  | 

## Example

```python
from tachyon_sdk.models.agent_chunk_event_one_of import AgentChunkEventOneOf

# TODO update the JSON string below
json = "{}"
# create an instance of AgentChunkEventOneOf from a JSON string
agent_chunk_event_one_of_instance = AgentChunkEventOneOf.from_json(json)
# print the JSON string representation of the object
print(AgentChunkEventOneOf.to_json())

# convert the object into a dict
agent_chunk_event_one_of_dict = agent_chunk_event_one_of_instance.to_dict()
# create an instance of AgentChunkEventOneOf from a dict
agent_chunk_event_one_of_from_dict = AgentChunkEventOneOf.from_dict(agent_chunk_event_one_of_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


