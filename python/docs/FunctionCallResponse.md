# FunctionCallResponse

Function call response type

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**arguments** | **str** | The arguments passed to the function | 
**name** | **str** | The name of the function that was called | 

## Example

```python
from tachyon_sdk.models.function_call_response import FunctionCallResponse

# TODO update the JSON string below
json = "{}"
# create an instance of FunctionCallResponse from a JSON string
function_call_response_instance = FunctionCallResponse.from_json(json)
# print the JSON string representation of the object
print(FunctionCallResponse.to_json())

# convert the object into a dict
function_call_response_dict = function_call_response_instance.to_dict()
# create an instance of FunctionCallResponse from a dict
function_call_response_from_dict = FunctionCallResponse.from_dict(function_call_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


