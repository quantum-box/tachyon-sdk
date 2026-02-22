# CreateOperatorRequest

Request body for creating an operator

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**new_operator_owner_id** | **str** |  | 
**new_operator_owner_method** | **str** |  | 
**new_operator_owner_password** | **str** |  | [optional] 
**operator_alias** | **str** |  | [optional] 
**operator_name** | **str** |  | 
**platform_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.create_operator_request import CreateOperatorRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateOperatorRequest from a JSON string
create_operator_request_instance = CreateOperatorRequest.from_json(json)
# print the JSON string representation of the object
print(CreateOperatorRequest.to_json())

# convert the object into a dict
create_operator_request_dict = create_operator_request_instance.to_dict()
# create an instance of CreateOperatorRequest from a dict
create_operator_request_from_dict = CreateOperatorRequest.from_dict(create_operator_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


