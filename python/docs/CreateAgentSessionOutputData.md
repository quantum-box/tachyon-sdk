# CreateAgentSessionOutputData

Output after session creation.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**name** | **str** |  | [optional] 
**session_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.create_agent_session_output_data import CreateAgentSessionOutputData

# TODO update the JSON string below
json = "{}"
# create an instance of CreateAgentSessionOutputData from a JSON string
create_agent_session_output_data_instance = CreateAgentSessionOutputData.from_json(json)
# print the JSON string representation of the object
print(CreateAgentSessionOutputData.to_json())

# convert the object into a dict
create_agent_session_output_data_dict = create_agent_session_output_data_instance.to_dict()
# create an instance of CreateAgentSessionOutputData from a dict
create_agent_session_output_data_from_dict = CreateAgentSessionOutputData.from_dict(create_agent_session_output_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


