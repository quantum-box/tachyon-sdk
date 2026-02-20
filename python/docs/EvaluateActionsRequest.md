# EvaluateActionsRequest

Request body: action permission evaluation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**actions** | [**List[ActionEvaluationRequest]**](ActionEvaluationRequest.md) | List of actions to evaluate | [optional] 

## Example

```python
from tachyon_sdk.models.evaluate_actions_request import EvaluateActionsRequest

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluateActionsRequest from a JSON string
evaluate_actions_request_instance = EvaluateActionsRequest.from_json(json)
# print the JSON string representation of the object
print(EvaluateActionsRequest.to_json())

# convert the object into a dict
evaluate_actions_request_dict = evaluate_actions_request_instance.to_dict()
# create an instance of EvaluateActionsRequest from a dict
evaluate_actions_request_from_dict = EvaluateActionsRequest.from_dict(evaluate_actions_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


