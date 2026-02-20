# AgentStatusResponse

Response for getting agent status

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**is_running** | **bool** | TODO: add English documentation | 
**progress** | **int** | TODO: add English documentation | 
**state** | **str** | TODO: add English documentation | 

## Example

```python
from tachyon_sdk.models.agent_status_response import AgentStatusResponse

# TODO update the JSON string below
json = "{}"
# create an instance of AgentStatusResponse from a JSON string
agent_status_response_instance = AgentStatusResponse.from_json(json)
# print the JSON string representation of the object
print(AgentStatusResponse.to_json())

# convert the object into a dict
agent_status_response_dict = agent_status_response_instance.to_dict()
# create an instance of AgentStatusResponse from a dict
agent_status_response_from_dict = AgentStatusResponse.from_dict(agent_status_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


