# ServiceAccountListResponse

Response for service account list

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**service_accounts** | [**List[ServiceAccountResponse]**](ServiceAccountResponse.md) |  | 

## Example

```python
from tachyon_sdk.models.service_account_list_response import ServiceAccountListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ServiceAccountListResponse from a JSON string
service_account_list_response_instance = ServiceAccountListResponse.from_json(json)
# print the JSON string representation of the object
print(ServiceAccountListResponse.to_json())

# convert the object into a dict
service_account_list_response_dict = service_account_list_response_instance.to_dict()
# create an instance of ServiceAccountListResponse from a dict
service_account_list_response_from_dict = ServiceAccountListResponse.from_dict(service_account_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


