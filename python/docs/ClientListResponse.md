# ClientListResponse

Client list response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[ClientResponse]**](ClientResponse.md) | Client items | 

## Example

```python
from tachyon_sdk.models.client_list_response import ClientListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ClientListResponse from a JSON string
client_list_response_instance = ClientListResponse.from_json(json)
# print the JSON string representation of the object
print(ClientListResponse.to_json())

# convert the object into a dict
client_list_response_dict = client_list_response_instance.to_dict()
# create an instance of ClientListResponse from a dict
client_list_response_from_dict = ClientListResponse.from_dict(client_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


