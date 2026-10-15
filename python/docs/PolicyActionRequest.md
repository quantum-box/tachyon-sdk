# PolicyActionRequest

Policy action entry

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action_full_name** | **str** | Action full name | 
**effect** | **str** | Policy effect. Supported values are \&quot;allow\&quot; and \&quot;deny\&quot; | 

## Example

```python
from tachyon_sdk.models.policy_action_request import PolicyActionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of PolicyActionRequest from a JSON string
policy_action_request_instance = PolicyActionRequest.from_json(json)
# print the JSON string representation of the object
print(PolicyActionRequest.to_json())

# convert the object into a dict
policy_action_request_dict = policy_action_request_instance.to_dict()
# create an instance of PolicyActionRequest from a dict
policy_action_request_from_dict = PolicyActionRequest.from_dict(policy_action_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


