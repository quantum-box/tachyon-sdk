# ActionListResponse

Response for action list

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**actions** | [**List[ActionResponse]**](ActionResponse.md) |  | 
**total_count** | **int** |  | 

## Example

```python
from tachyon_sdk.models.action_list_response import ActionListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ActionListResponse from a JSON string
action_list_response_instance = ActionListResponse.from_json(json)
# print the JSON string representation of the object
print(ActionListResponse.to_json())

# convert the object into a dict
action_list_response_dict = action_list_response_instance.to_dict()
# create an instance of ActionListResponse from a dict
action_list_response_from_dict = ActionListResponse.from_dict(action_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


