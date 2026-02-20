# CreateServiceAccountRequest

Request to create a service account

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | Display name | 
**tenant_id** | **str** | Tenant ID the service account belongs to | 

## Example

```python
from tachyon_sdk.models.create_service_account_request import CreateServiceAccountRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateServiceAccountRequest from a JSON string
create_service_account_request_instance = CreateServiceAccountRequest.from_json(json)
# print the JSON string representation of the object
print(CreateServiceAccountRequest.to_json())

# convert the object into a dict
create_service_account_request_dict = create_service_account_request_instance.to_dict()
# create an instance of CreateServiceAccountRequest from a dict
create_service_account_request_from_dict = CreateServiceAccountRequest.from_dict(create_service_account_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


