# RegisterPolicyRequest

Request to register a custom policy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action_patterns** | [**List[PolicyActionPatternRequest]**](PolicyActionPatternRequest.md) |  | [optional] [default to []]
**actions** | [**List[PolicyActionRequest]**](PolicyActionRequest.md) |  | [optional] [default to []]
**description** | **str** | Policy description | [optional] 
**var_global** | **bool** |  | [optional] [default to False]
**is_system** | **bool** |  | [optional] [default to False]
**name** | **str** | Policy name | 
**tenant_id** | **str** | Tenant ID | [optional] 

## Example

```python
from tachyon_sdk.models.register_policy_request import RegisterPolicyRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RegisterPolicyRequest from a JSON string
register_policy_request_instance = RegisterPolicyRequest.from_json(json)
# print the JSON string representation of the object
print(RegisterPolicyRequest.to_json())

# convert the object into a dict
register_policy_request_dict = register_policy_request_instance.to_dict()
# create an instance of RegisterPolicyRequest from a dict
register_policy_request_from_dict = RegisterPolicyRequest.from_dict(register_policy_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


