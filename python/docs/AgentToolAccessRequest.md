# AgentToolAccessRequest

Builtin tools enabled for agent execution. Omitted tools are disabled.

## Type

```python
list[dict[str, str]]
```

Each item has:

Name | Type | Description
------------ | ------------- | -------------
**type** | **str** | Must be `builtin`
**name** | **str** | Builtin tool name

## Example

```python
from tachyon_sdk.models.agent_tool_access_request import AgentToolAccessRequest

tools = AgentToolAccessRequest.from_dict([
    {"type": "builtin", "name": "filesystem"},
    {"type": "builtin", "name": "web_search"},
    {"type": "builtin", "name": "url_fetch"},
])

print(tools.to_json())
print(tools.to_dict())
```

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
