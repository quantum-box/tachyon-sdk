# CreateAgentSessionRequest

Request body for creating a new agent session.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**metadata** | **object** |  | [optional] 
**name** | **str** | Optional display name for the session. | [optional] 

## Example

```python
from tachyon_sdk.models.create_agent_session_request import CreateAgentSessionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateAgentSessionRequest from a JSON string
create_agent_session_request_instance = CreateAgentSessionRequest.from_json(json)
# print the JSON string representation of the object
print(CreateAgentSessionRequest.to_json())

# convert the object into a dict
create_agent_session_request_dict = create_agent_session_request_instance.to_dict()
# create an instance of CreateAgentSessionRequest from a dict
create_agent_session_request_from_dict = CreateAgentSessionRequest.from_dict(create_agent_session_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


