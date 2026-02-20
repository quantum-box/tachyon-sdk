# AgentChunk

A streaming chunk with optional agent metadata.  When `agent` is `None`, the chunk originates from the main agent. When `Some`, it was relayed from a sub-agent.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tool_id** | **str** | Unique identifier for this tool call. Use this value as &#x60;tool_id&#x60; when submitting the result. | 
**tool_name** | **str** | Name of the client-defined tool the LLM wants to call. | 
**type** | **str** |  | 
**args** | **object** |  | 
**is_finished** | **bool** |  | 
**result** | **str** |  | 
**fire_and_forget** | **bool** | When &#x60;true&#x60;, the server does not wait for the client to submit a tool result — the LLM continues immediately. The client may still execute the tool for its side effects. | [optional] 
**index** | **int** |  | 
**text** | **str** |  | 
**created_at** | **datetime** |  | 
**id** | **str** |  | 
**user_id** | **str** |  | 
**options** | **List[str]** |  | 
**command** | **str** |  | [optional] 
**cache_creation_input_tokens** | **int** |  | [optional] 
**cache_read_input_tokens** | **int** |  | [optional] 
**completion_tokens** | **int** |  | 
**prompt_tokens** | **int** |  | 
**total_cost** | **float** |  | [optional] 
**total_tokens** | **int** |  | 
**agent** | [**AgentSource**](AgentSource.md) |  | [optional] 

## Example

```python
from tachyon_sdk.models.agent_chunk import AgentChunk

# TODO update the JSON string below
json = "{}"
# create an instance of AgentChunk from a JSON string
agent_chunk_instance = AgentChunk.from_json(json)
# print the JSON string representation of the object
print(AgentChunk.to_json())

# convert the object into a dict
agent_chunk_dict = agent_chunk_instance.to_dict()
# create an instance of AgentChunk from a dict
agent_chunk_from_dict = AgentChunk.from_dict(agent_chunk_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


