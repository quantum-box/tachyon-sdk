# GetAgentSessionOutputData

Output containing the session detail.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**id** | **str** |  | 
**metadata** | **object** |  | [optional] 
**name** | **str** |  | [optional] 
**updated_at** | **datetime** |  | 

## Example

```python
from tachyon_sdk.models.get_agent_session_output_data import GetAgentSessionOutputData

# TODO update the JSON string below
json = "{}"
# create an instance of GetAgentSessionOutputData from a JSON string
get_agent_session_output_data_instance = GetAgentSessionOutputData.from_json(json)
# print the JSON string representation of the object
print(GetAgentSessionOutputData.to_json())

# convert the object into a dict
get_agent_session_output_data_dict = get_agent_session_output_data_instance.to_dict()
# create an instance of GetAgentSessionOutputData from a dict
get_agent_session_output_data_from_dict = GetAgentSessionOutputData.from_dict(get_agent_session_output_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


