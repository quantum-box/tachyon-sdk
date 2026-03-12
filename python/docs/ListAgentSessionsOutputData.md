# ListAgentSessionsOutputData

Output containing the list of sessions.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**sessions** | [**List[AgentSessionEntry]**](AgentSessionEntry.md) |  | 

## Example

```python
from tachyon_sdk.models.list_agent_sessions_output_data import ListAgentSessionsOutputData

# TODO update the JSON string below
json = "{}"
# create an instance of ListAgentSessionsOutputData from a JSON string
list_agent_sessions_output_data_instance = ListAgentSessionsOutputData.from_json(json)
# print the JSON string representation of the object
print(ListAgentSessionsOutputData.to_json())

# convert the object into a dict
list_agent_sessions_output_data_dict = list_agent_sessions_output_data_instance.to_dict()
# create an instance of ListAgentSessionsOutputData from a dict
list_agent_sessions_output_data_from_dict = ListAgentSessionsOutputData.from_dict(list_agent_sessions_output_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


