# EvaluatePoliciesBatchResponse

Response for batch policy evaluation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**results** | [**List[PolicyEvaluationOutcome]**](PolicyEvaluationOutcome.md) |  | 

## Example

```python
from tachyon_sdk.models.evaluate_policies_batch_response import EvaluatePoliciesBatchResponse

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluatePoliciesBatchResponse from a JSON string
evaluate_policies_batch_response_instance = EvaluatePoliciesBatchResponse.from_json(json)
# print the JSON string representation of the object
print(EvaluatePoliciesBatchResponse.to_json())

# convert the object into a dict
evaluate_policies_batch_response_dict = evaluate_policies_batch_response_instance.to_dict()
# create an instance of EvaluatePoliciesBatchResponse from a dict
evaluate_policies_batch_response_from_dict = EvaluatePoliciesBatchResponse.from_dict(evaluate_policies_batch_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


