# PolicyEvaluationOutcome

Single outcome of a batch policy evaluation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | **str** |  | 
**allowed** | **bool** |  | 
**error** | **str** |  | [optional] 

## Example

```python
from tachyon_sdk.models.policy_evaluation_outcome import PolicyEvaluationOutcome

# TODO update the JSON string below
json = "{}"
# create an instance of PolicyEvaluationOutcome from a JSON string
policy_evaluation_outcome_instance = PolicyEvaluationOutcome.from_json(json)
# print the JSON string representation of the object
print(PolicyEvaluationOutcome.to_json())

# convert the object into a dict
policy_evaluation_outcome_dict = policy_evaluation_outcome_instance.to_dict()
# create an instance of PolicyEvaluationOutcome from a dict
policy_evaluation_outcome_from_dict = PolicyEvaluationOutcome.from_dict(policy_evaluation_outcome_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


