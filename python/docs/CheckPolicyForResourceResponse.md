# CheckPolicyForResourceResponse

Response for resource-level policy check

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allowed** | **bool** |  | 

## Example

```python
from tachyon_sdk.models.check_policy_for_resource_response import CheckPolicyForResourceResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CheckPolicyForResourceResponse from a JSON string
check_policy_for_resource_response_instance = CheckPolicyForResourceResponse.from_json(json)
# print the JSON string representation of the object
print(CheckPolicyForResourceResponse.to_json())

# convert the object into a dict
check_policy_for_resource_response_dict = check_policy_for_resource_response_instance.to_dict()
# create an instance of CheckPolicyForResourceResponse from a dict
check_policy_for_resource_response_from_dict = CheckPolicyForResourceResponse.from_dict(check_policy_for_resource_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


