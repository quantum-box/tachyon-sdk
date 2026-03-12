# UserPolicyListResponse

Response for user policy list

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**policy_ids** | **List[str]** |  | 

## Example

```python
from tachyon_sdk.models.user_policy_list_response import UserPolicyListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of UserPolicyListResponse from a JSON string
user_policy_list_response_instance = UserPolicyListResponse.from_json(json)
# print the JSON string representation of the object
print(UserPolicyListResponse.to_json())

# convert the object into a dict
user_policy_list_response_dict = user_policy_list_response_instance.to_dict()
# create an instance of UserPolicyListResponse from a dict
user_policy_list_response_from_dict = UserPolicyListResponse.from_dict(user_policy_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


