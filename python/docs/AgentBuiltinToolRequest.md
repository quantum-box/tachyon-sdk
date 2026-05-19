# AgentBuiltinToolRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** |  | 
**type** | [**AgentBuiltinToolType**](AgentBuiltinToolType.md) |  | 

## Example

```python
from tachyon_sdk.models.agent_builtin_tool_request import AgentBuiltinToolRequest

# TODO update the JSON string below
json = "{}"
# create an instance of AgentBuiltinToolRequest from a JSON string
agent_builtin_tool_request_instance = AgentBuiltinToolRequest.from_json(json)
# print the JSON string representation of the object
print(AgentBuiltinToolRequest.to_json())

# convert the object into a dict
agent_builtin_tool_request_dict = agent_builtin_tool_request_instance.to_dict()
# create an instance of AgentBuiltinToolRequest from a dict
agent_builtin_tool_request_from_dict = AgentBuiltinToolRequest.from_dict(agent_builtin_tool_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


