# DeleteServiceAccountResponse

Response for delete operation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.delete_service_account_response import DeleteServiceAccountResponse

# TODO update the JSON string below
json = "{}"
# create an instance of DeleteServiceAccountResponse from a JSON string
delete_service_account_response_instance = DeleteServiceAccountResponse.from_json(json)
# print the JSON string representation of the object
print(DeleteServiceAccountResponse.to_json())

# convert the object into a dict
delete_service_account_response_dict = delete_service_account_response_instance.to_dict()
# create an instance of DeleteServiceAccountResponse from a dict
delete_service_account_response_from_dict = DeleteServiceAccountResponse.from_dict(delete_service_account_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


