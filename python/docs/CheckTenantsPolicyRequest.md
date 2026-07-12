# CheckTenantsPolicyRequest

Request to evaluate one action across multiple tenant scopes.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | **str** |  |
**platform_id** | **str** |  |
**tenant_ids** | **List[str]** |  | [optional]

## Example

```python
from tachyon_sdk.models.check_tenants_policy_request import CheckTenantsPolicyRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CheckTenantsPolicyRequest from a JSON string
check_tenants_policy_request_instance = CheckTenantsPolicyRequest.from_json(json)
# print the JSON string representation of the object
print(CheckTenantsPolicyRequest.to_json())

# convert the object into a dict
check_tenants_policy_request_dict = check_tenants_policy_request_instance.to_dict()
# create an instance of CheckTenantsPolicyRequest from a dict
check_tenants_policy_request_from_dict = CheckTenantsPolicyRequest.from_dict(check_tenants_policy_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
