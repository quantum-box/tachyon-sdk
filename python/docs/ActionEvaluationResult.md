# ActionEvaluationResult

Evaluation result for each action

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | **str** | Target action | 
**context** | **str** | Context used for evaluation (tenant info, etc.) | 
**feature_enabled** | **bool** | Whether Feature Flag is enabled | 
**feature_error** | **str** | Error message during Feature Flag evaluation | [optional] 
**policy_allowed** | **bool** | Whether allowed by Policy | 
**policy_error** | **str** | Error message during Policy evaluation | [optional] 

## Example

```python
from tachyon_sdk.models.action_evaluation_result import ActionEvaluationResult

# TODO update the JSON string below
json = "{}"
# create an instance of ActionEvaluationResult from a JSON string
action_evaluation_result_instance = ActionEvaluationResult.from_json(json)
# print the JSON string representation of the object
print(ActionEvaluationResult.to_json())

# convert the object into a dict
action_evaluation_result_dict = action_evaluation_result_instance.to_dict()
# create an instance of ActionEvaluationResult from a dict
action_evaluation_result_from_dict = ActionEvaluationResult.from_dict(action_evaluation_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


