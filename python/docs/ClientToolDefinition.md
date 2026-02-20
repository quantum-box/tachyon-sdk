# ClientToolDefinition

User-defined tool definition sent via the API request.  Uses JSON Schema format compatible with OpenAI/Anthropic function calling. Tool definitions are passed to the LLM alongside server built-in tools so the model can decide when to invoke them.  # Example  ```json {   \"name\": \"query_database\",   \"description\": \"Run a read-only SQL query\",   \"parameters\": {     \"type\": \"object\",     \"properties\": {       \"sql\": { \"type\": \"string\" }     },     \"required\": [\"sql\"]   },   \"fire_and_forget\": false } ```

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | **str** | Human-readable description shown to the LLM so it can decide when to call this tool. | 
**fire_and_forget** | **bool** | When &#x60;true&#x60;, the server dispatches the tool call to the client and immediately returns a success message to the LLM without waiting for a result submission. The client still receives the &#x60;tool_call_pending&#x60; SSE event but does not need to call the tool-result endpoint. Useful for notifications, webhooks, logging, and other side-effect-only operations. | [optional] 
**name** | **str** | Unique tool name. Must not collide with server built-in tools (e.g. &#x60;read_file&#x60;, &#x60;execute_command&#x60;, &#x60;search_files&#x60;). Use descriptive, snake_case names. | 
**parameters** | **object** |  | 

## Example

```python
from tachyon_sdk.models.client_tool_definition import ClientToolDefinition

# TODO update the JSON string below
json = "{}"
# create an instance of ClientToolDefinition from a JSON string
client_tool_definition_instance = ClientToolDefinition.from_json(json)
# print the JSON string representation of the object
print(ClientToolDefinition.to_json())

# convert the object into a dict
client_tool_definition_dict = client_tool_definition_instance.to_dict()
# create an instance of ClientToolDefinition from a dict
client_tool_definition_from_dict = ClientToolDefinition.from_dict(client_tool_definition_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


