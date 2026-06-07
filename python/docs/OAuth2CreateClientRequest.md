# OAuth2CreateClientRequest

Request to create an OAuth2 client

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allowed_scopes** | **List[str]** | Allowed scopes |
**auth_mode** | **str** | Authentication mode: \&quot;direct\&quot; or \&quot;proxied\&quot; | [optional]
**grant_types** | **List[str]** | Allowed grant types |
**name** | **str** | Display name |
**redirect_uris** | **List[str]** | Allowed redirect URIs |
**use_tachyon_user_pool** | **bool** | Whether to use Tachyon user pool | [optional] [default to True]
**user_pool_id** | **str** | Associated User Pool ID when using a dedicated pool | [optional]

## Example

```python
from tachyon_sdk.models.o_auth2_create_client_request import OAuth2CreateClientRequest

# TODO update the JSON string below
json = "{}"
# create an instance of OAuth2CreateClientRequest from a JSON string
o_auth2_create_client_request_instance = OAuth2CreateClientRequest.from_json(json)
# print the JSON string representation of the object
print(OAuth2CreateClientRequest.to_json())

# convert the object into a dict
o_auth2_create_client_request_dict = o_auth2_create_client_request_instance.to_dict()
# create an instance of OAuth2CreateClientRequest from a dict
o_auth2_create_client_request_from_dict = OAuth2CreateClientRequest.from_dict(o_auth2_create_client_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


