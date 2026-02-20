# AgentChunkEventOneOf3

Tool call pending client-side execution

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**args** | **object** |  | 
**fire_and_forget** | **bool** | When &#x60;true&#x60;, the server does not wait for the client to submit a tool result — the LLM continues immediately. The client may still execute the tool for its side effects. | [optional] 
**tool_id** | **str** | Unique identifier for this tool call. Use this value as &#x60;tool_id&#x60; when submitting the result. | 
**tool_name** | **str** | Name of the client-defined tool the LLM wants to call. | 
**type** | **str** |  | 

## Example

```python
from tachyon_sdk.models.agent_chunk_event_one_of3 import AgentChunkEventOneOf3

# TODO update the JSON string below
json = "{}"
# create an instance of AgentChunkEventOneOf3 from a JSON string
agent_chunk_event_one_of3_instance = AgentChunkEventOneOf3.from_json(json)
# print the JSON string representation of the object
print(AgentChunkEventOneOf3.to_json())

# convert the object into a dict
agent_chunk_event_one_of3_dict = agent_chunk_event_one_of3_instance.to_dict()
# create an instance of AgentChunkEventOneOf3 from a dict
agent_chunk_event_one_of3_from_dict = AgentChunkEventOneOf3.from_dict(agent_chunk_event_one_of3_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


