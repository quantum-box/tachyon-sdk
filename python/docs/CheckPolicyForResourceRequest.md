# CheckPolicyForResourceRequest

Request to check policy for a specific resource

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | **str** | Action name (e.g., \&quot;library:UpdateRepo\&quot;) | 
**resource_trn** | **str** | Target resource TRN (e.g., \&quot;trn:library:repo:rp_xxx\&quot;) | 

## Example

```python
from tachyon_sdk.models.check_policy_for_resource_request import CheckPolicyForResourceRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CheckPolicyForResourceRequest from a JSON string
check_policy_for_resource_request_instance = CheckPolicyForResourceRequest.from_json(json)
# print the JSON string representation of the object
print(CheckPolicyForResourceRequest.to_json())

# convert the object into a dict
check_policy_for_resource_request_dict = check_policy_for_resource_request_instance.to_dict()
# create an instance of CheckPolicyForResourceRequest from a dict
check_policy_for_resource_request_from_dict = CheckPolicyForResourceRequest.from_dict(check_policy_for_resource_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


