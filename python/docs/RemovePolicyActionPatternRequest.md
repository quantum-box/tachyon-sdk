# RemovePolicyActionPatternRequest

Policy action pattern removal entry

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**context_pattern** | **str** | Action context pattern |
**name_pattern** | **str** | Action name pattern |

## Example

```python
from tachyon_sdk.models.remove_policy_action_pattern_request import RemovePolicyActionPatternRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RemovePolicyActionPatternRequest from a JSON string
remove_policy_action_pattern_request_instance = RemovePolicyActionPatternRequest.from_json(json)
# print the JSON string representation of the object
print(RemovePolicyActionPatternRequest.to_json())

# convert the object into a dict
remove_policy_action_pattern_request_dict = remove_policy_action_pattern_request_instance.to_dict()
# create an instance of RemovePolicyActionPatternRequest from a dict
remove_policy_action_pattern_request_from_dict = RemovePolicyActionPatternRequest.from_dict(remove_policy_action_pattern_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


