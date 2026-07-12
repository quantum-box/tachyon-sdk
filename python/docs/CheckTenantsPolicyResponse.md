# CheckTenantsPolicyResponse

Response for tenant-bulk policy evaluation.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allowed_tenants** | [**List[AllowedTenantResponse]**](AllowedTenantResponse.md) |  |

## Example

```python
from tachyon_sdk.models.check_tenants_policy_response import CheckTenantsPolicyResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CheckTenantsPolicyResponse from a JSON string
check_tenants_policy_response_instance = CheckTenantsPolicyResponse.from_json(json)
# print the JSON string representation of the object
print(CheckTenantsPolicyResponse.to_json())

# convert the object into a dict
check_tenants_policy_response_dict = check_tenants_policy_response_instance.to_dict()
# create an instance of CheckTenantsPolicyResponse from a dict
check_tenants_policy_response_from_dict = CheckTenantsPolicyResponse.from_dict(check_tenants_policy_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
