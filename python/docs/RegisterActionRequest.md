# RegisterActionRequest

Request to register a custom action

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**context** | **str** | Action context | 
**description** | **str** | Description | [optional] 
**name** | **str** | Action name | 
**resource_pattern** | **str** | Resource pattern | [optional] 
**sandbox_restriction** | **str** | Sandbox restriction: \&quot;allow\&quot;, \&quot;deny\&quot;, or \&quot;audit\&quot; | [optional] 

## Example

```python
from tachyon_sdk.models.register_action_request import RegisterActionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RegisterActionRequest from a JSON string
register_action_request_instance = RegisterActionRequest.from_json(json)
# print the JSON string representation of the object
print(RegisterActionRequest.to_json())

# convert the object into a dict
register_action_request_dict = register_action_request_instance.to_dict()
# create an instance of RegisterActionRequest from a dict
register_action_request_from_dict = RegisterActionRequest.from_dict(register_action_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


