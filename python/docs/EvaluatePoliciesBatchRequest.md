# EvaluatePoliciesBatchRequest

Request to evaluate policies in batch

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**actions** | **List[str]** | List of action names to check | 

## Example

```python
from tachyon_sdk.models.evaluate_policies_batch_request import EvaluatePoliciesBatchRequest

# TODO update the JSON string below
json = "{}"
# create an instance of EvaluatePoliciesBatchRequest from a JSON string
evaluate_policies_batch_request_instance = EvaluatePoliciesBatchRequest.from_json(json)
# print the JSON string representation of the object
print(EvaluatePoliciesBatchRequest.to_json())

# convert the object into a dict
evaluate_policies_batch_request_dict = evaluate_policies_batch_request_instance.to_dict()
# create an instance of EvaluatePoliciesBatchRequest from a dict
evaluate_policies_batch_request_from_dict = EvaluatePoliciesBatchRequest.from_dict(evaluate_policies_batch_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


