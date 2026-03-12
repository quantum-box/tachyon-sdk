# AgentChunkEvent

Agent chunk event types for streaming responses.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**is_client_tool** | **bool** | When &#x60;true&#x60;, this tool call targets a client-defined tool. The client should handle it locally and submit the result via the tool-result endpoint (unless &#x60;fire_and_forget&#x60;). | [optional] 
**tool_id** | **str** |  | 
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
**job_id** | **str** |  | 
**provider** | **str** |  | 

## Example

```python
from tachyon_sdk.models.agent_chunk_event import AgentChunkEvent

# TODO update the JSON string below
json = "{}"
# create an instance of AgentChunkEvent from a JSON string
agent_chunk_event_instance = AgentChunkEvent.from_json(json)
# print the JSON string representation of the object
print(AgentChunkEvent.to_json())

# convert the object into a dict
agent_chunk_event_dict = agent_chunk_event_instance.to_dict()
# create an instance of AgentChunkEvent from a dict
agent_chunk_event_from_dict = AgentChunkEvent.from_dict(agent_chunk_event_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


