# ActionEvaluationRequest

Action evaluation input

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | **str** | Action ID registered in policy/feature flag | 
**resource_pattern** | **str** | Resource scope specification (wildcards allowed) | [optional] 

## Example

```python
from tachyon_sdk.models.action_evaluation_request import ActionEvaluationRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ActionEvaluationRequest from a JSON string
action_evaluation_request_instance = ActionEvaluationRequest.from_json(json)
# print the JSON string representation of the object
print(ActionEvaluationRequest.to_json())

# convert the object into a dict
action_evaluation_request_dict = action_evaluation_request_instance.to_dict()
# create an instance of ActionEvaluationRequest from a dict
action_evaluation_request_from_dict = ActionEvaluationRequest.from_dict(action_evaluation_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


