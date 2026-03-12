# AgentChunkEventOneOf10

Emitted when a tool job is created, before sync polling begins. Frontend can use the job_id to subscribe to the tool job's own SSE stream for real-time progress.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**job_id** | **str** |  | 
**provider** | **str** |  | 
**tool_id** | **str** |  | 
**type** | **str** |  | 

## Example

```python
from tachyon_sdk.models.agent_chunk_event_one_of10 import AgentChunkEventOneOf10

# TODO update the JSON string below
json = "{}"
# create an instance of AgentChunkEventOneOf10 from a JSON string
agent_chunk_event_one_of10_instance = AgentChunkEventOneOf10.from_json(json)
# print the JSON string representation of the object
print(AgentChunkEventOneOf10.to_json())

# convert the object into a dict
agent_chunk_event_one_of10_dict = agent_chunk_event_one_of10_instance.to_dict()
# create an instance of AgentChunkEventOneOf10 from a dict
agent_chunk_event_one_of10_from_dict = AgentChunkEventOneOf10.from_dict(agent_chunk_event_one_of10_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


