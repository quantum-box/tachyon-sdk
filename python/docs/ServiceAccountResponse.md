# ServiceAccountResponse

Response for a service account

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **str** |  | 
**id** | **str** |  | 
**name** | **str** |  | 
**tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.service_account_response import ServiceAccountResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ServiceAccountResponse from a JSON string
service_account_response_instance = ServiceAccountResponse.from_json(json)
# print the JSON string representation of the object
print(ServiceAccountResponse.to_json())

# convert the object into a dict
service_account_response_dict = service_account_response_instance.to_dict()
# create an instance of ServiceAccountResponse from a dict
service_account_response_from_dict = ServiceAccountResponse.from_dict(service_account_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


