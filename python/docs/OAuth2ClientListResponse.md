# OAuth2ClientListResponse

OAuth2 client list response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clients** | [**List[OAuth2ClientResponse]**](OAuth2ClientResponse.md) |  |

## Example

```python
from tachyon_sdk.models.o_auth2_client_list_response import OAuth2ClientListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of OAuth2ClientListResponse from a JSON string
o_auth2_client_list_response_instance = OAuth2ClientListResponse.from_json(json)
# print the JSON string representation of the object
print(OAuth2ClientListResponse.to_json())

# convert the object into a dict
o_auth2_client_list_response_dict = o_auth2_client_list_response_instance.to_dict()
# create an instance of OAuth2ClientListResponse from a dict
o_auth2_client_list_response_from_dict = OAuth2ClientListResponse.from_dict(o_auth2_client_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


