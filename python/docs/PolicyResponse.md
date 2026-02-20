# PolicyResponse

Response for a policy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **str** |  | 
**description** | **str** |  | [optional] 
**id** | **str** |  | 
**is_system** | **bool** |  | 
**name** | **str** |  | 
**tenant_id** | **str** |  | [optional] 
**updated_at** | **str** |  | 

## Example

```python
from tachyon_sdk.models.policy_response import PolicyResponse

# TODO update the JSON string below
json = "{}"
# create an instance of PolicyResponse from a JSON string
policy_response_instance = PolicyResponse.from_json(json)
# print the JSON string representation of the object
print(PolicyResponse.to_json())

# convert the object into a dict
policy_response_dict = policy_response_instance.to_dict()
# create an instance of PolicyResponse from a dict
policy_response_from_dict = PolicyResponse.from_dict(policy_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


