# ToolCallPending

SSE event indicating a client-side tool call is pending.  Emitted when the LLM invokes a user-defined client tool. The client should execute the tool using the provided `args` and submit the result via the tool-result endpoint, unless `fire_and_forget` is `true`.  SSE event name: `tool_call_pending`

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**args** | **object** |  | 
**fire_and_forget** | **bool** | When &#x60;true&#x60;, the server does not wait for the client to submit a tool result — the LLM continues immediately. The client may still execute the tool for its side effects. | [optional] 
**tool_id** | **str** | Unique identifier for this tool call. Use this value as &#x60;tool_id&#x60; when submitting the result. | 
**tool_name** | **str** | Name of the client-defined tool the LLM wants to call. | 

## Example

```python
from tachyon_sdk.models.tool_call_pending import ToolCallPending

# TODO update the JSON string below
json = "{}"
# create an instance of ToolCallPending from a JSON string
tool_call_pending_instance = ToolCallPending.from_json(json)
# print the JSON string representation of the object
print(ToolCallPending.to_json())

# convert the object into a dict
tool_call_pending_dict = tool_call_pending_instance.to_dict()
# create an instance of ToolCallPending from a dict
tool_call_pending_from_dict = ToolCallPending.from_dict(tool_call_pending_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


