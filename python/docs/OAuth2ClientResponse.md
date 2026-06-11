# OAuth2ClientResponse

Response for an OAuth2 client (no secret)

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allowed_scopes** | **List[str]** | Allowed scopes |
**auth_mode** | **str** | Authentication mode | [optional]
**client_id** | **str** |  |
**created_at** | **str** |  |
**grant_types** | **List[str]** | Allowed grant types |
**id** | **str** |  |
**name** | **str** |  |
**redirect_uris** | **List[str]** | Allowed redirect URIs |
**status** | **str** |  |
**updated_at** | **str** |  |
**use_tachyon_user_pool** | **bool** |  |
**user_pool_id** | **str** |  | [optional]

## Example

```python
from tachyon_sdk.models.o_auth2_client_response import OAuth2ClientResponse

# TODO update the JSON string below
json = "{}"
# create an instance of OAuth2ClientResponse from a JSON string
o_auth2_client_response_instance = OAuth2ClientResponse.from_json(json)
# print the JSON string representation of the object
print(OAuth2ClientResponse.to_json())

# convert the object into a dict
o_auth2_client_response_dict = o_auth2_client_response_instance.to_dict()
# create an instance of OAuth2ClientResponse from a dict
o_auth2_client_response_from_dict = OAuth2ClientResponse.from_dict(o_auth2_client_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


