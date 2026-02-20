# AttemptCompletionResult

Task completion result

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**command** | **str** |  | [optional] 
**is_finished** | **bool** |  | [optional] 
**result** | **str** |  | 

## Example

```python
from tachyon_sdk.models.attempt_completion_result import AttemptCompletionResult

# TODO update the JSON string below
json = "{}"
# create an instance of AttemptCompletionResult from a JSON string
attempt_completion_result_instance = AttemptCompletionResult.from_json(json)
# print the JSON string representation of the object
print(AttemptCompletionResult.to_json())

# convert the object into a dict
attempt_completion_result_dict = attempt_completion_result_instance.to_dict()
# create an instance of AttemptCompletionResult from a dict
attempt_completion_result_from_dict = AttemptCompletionResult.from_dict(attempt_completion_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


