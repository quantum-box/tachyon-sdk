# AgentSessionEntry

Single session entry in the list response.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**id** | **str** |  | 
**name** | **str** |  | [optional] 
**updated_at** | **datetime** |  | 

## Example

```python
from tachyon_sdk.models.agent_session_entry import AgentSessionEntry

# TODO update the JSON string below
json = "{}"
# create an instance of AgentSessionEntry from a JSON string
agent_session_entry_instance = AgentSessionEntry.from_json(json)
# print the JSON string representation of the object
print(AgentSessionEntry.to_json())

# convert the object into a dict
agent_session_entry_dict = agent_session_entry_instance.to_dict()
# create an instance of AgentSessionEntry from a dict
agent_session_entry_from_dict = AgentSessionEntry.from_dict(agent_session_entry_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


