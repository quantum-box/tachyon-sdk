# SaveOAuthTokenRequest

Request to save an OAuth token

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**access_token** | **str** | Access token | 
**expires_in** | **int** | Token expiration in seconds | 
**provider** | **str** | OAuth provider name (e.g., \&quot;github\&quot;) | 
**provider_user_id** | **str** | Provider user ID | 
**refresh_token** | **str** | Refresh token (optional) | [optional] 
**scope** | **str** | OAuth scope (optional) | [optional] 

## Example

```python
from tachyon_sdk.models.save_o_auth_token_request import SaveOAuthTokenRequest

# TODO update the JSON string below
json = "{}"
# create an instance of SaveOAuthTokenRequest from a JSON string
save_o_auth_token_request_instance = SaveOAuthTokenRequest.from_json(json)
# print the JSON string representation of the object
print(SaveOAuthTokenRequest.to_json())

# convert the object into a dict
save_o_auth_token_request_dict = save_o_auth_token_request_instance.to_dict()
# create an instance of SaveOAuthTokenRequest from a dict
save_o_auth_token_request_from_dict = SaveOAuthTokenRequest.from_dict(save_o_auth_token_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


