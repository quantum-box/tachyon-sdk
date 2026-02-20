# EvaluateActionsResponse

Response body

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**results** | [**List[ActionEvaluationResult]**](ActionEvaluationResult.md) | Evaluation result for each action | 

## Example

```python
from tachyon_sdk.models.evaluate_actions_response import EvaluateActionsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluateActionsResponse from a JSON string
evaluate_actions_response_instance = EvaluateActionsResponse.from_json(json)
# print the JSON string representation of the object
print(EvaluateActionsResponse.to_json())

# convert the object into a dict
evaluate_actions_response_dict = evaluate_actions_response_instance.to_dict()
# create an instance of EvaluateActionsResponse from a dict
evaluate_actions_response_from_dict = EvaluateActionsResponse.from_dict(evaluate_actions_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


