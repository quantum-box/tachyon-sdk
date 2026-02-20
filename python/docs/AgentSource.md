# AgentSource

Identifies which agent produced a chunk.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chatroom_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.agent_source import AgentSource

# TODO update the JSON string below
json = "{}"
# create an instance of AgentSource from a JSON string
agent_source_instance = AgentSource.from_json(json)
# print the JSON string representation of the object
print(AgentSource.to_json())

# convert the object into a dict
agent_source_dict = agent_source_instance.to_dict()
# create an instance of AgentSource from a dict
agent_source_from_dict = AgentSource.from_dict(agent_source_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


