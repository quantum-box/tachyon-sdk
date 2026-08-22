# PolicyActionPatternRequest

Policy action pattern entry

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**context_pattern** | **str** | Action context pattern | 
**effect** | **str** | Policy effect. Supported values are \&quot;allow\&quot; and \&quot;deny\&quot; | 
**name_pattern** | **str** | Action name pattern | 

## Example

```python
from tachyon_sdk.models.policy_action_pattern_request import PolicyActionPatternRequest

# TODO update the JSON string below
json = "{}"
# create an instance of PolicyActionPatternRequest from a JSON string
policy_action_pattern_request_instance = PolicyActionPatternRequest.from_json(json)
# print the JSON string representation of the object
print(PolicyActionPatternRequest.to_json())

# convert the object into a dict
policy_action_pattern_request_dict = policy_action_pattern_request_instance.to_dict()
# create an instance of PolicyActionPatternRequest from a dict
policy_action_pattern_request_from_dict = PolicyActionPatternRequest.from_dict(policy_action_pattern_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


