# CreateOperatorResponse

Response for creating an operator

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operator** | [**OperatorResponse**](OperatorResponse.md) |  | 
**owner_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.create_operator_response import CreateOperatorResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CreateOperatorResponse from a JSON string
create_operator_response_instance = CreateOperatorResponse.from_json(json)
# print the JSON string representation of the object
print(CreateOperatorResponse.to_json())

# convert the object into a dict
create_operator_response_dict = create_operator_response_instance.to_dict()
# create an instance of CreateOperatorResponse from a dict
create_operator_response_from_dict = CreateOperatorResponse.from_dict(create_operator_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


