# DeleteOperatorResponse

Response for deleting an operator

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**success** | **bool** |  | 

## Example

```python
from tachyon_sdk.models.delete_operator_response import DeleteOperatorResponse

# TODO update the JSON string below
json = "{}"
# create an instance of DeleteOperatorResponse from a JSON string
delete_operator_response_instance = DeleteOperatorResponse.from_json(json)
# print the JSON string representation of the object
print(DeleteOperatorResponse.to_json())

# convert the object into a dict
delete_operator_response_dict = delete_operator_response_instance.to_dict()
# create an instance of DeleteOperatorResponse from a dict
delete_operator_response_from_dict = DeleteOperatorResponse.from_dict(delete_operator_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
