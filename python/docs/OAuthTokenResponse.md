# OAuthTokenResponse

Response for an OAuth token

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**access_token** | **str** |  | 
**expires_at** | **str** |  | 
**provider** | **str** |  | 
**provider_user_id** | **str** |  | 
**refresh_token** | **str** |  | [optional] 
**scope** | **str** |  | [optional] 
**token_type** | **str** |  | 

## Example

```python
from tachyon_sdk.models.o_auth_token_response import OAuthTokenResponse

# TODO update the JSON string below
json = "{}"
# create an instance of OAuthTokenResponse from a JSON string
o_auth_token_response_instance = OAuthTokenResponse.from_json(json)
# print the JSON string representation of the object
print(OAuthTokenResponse.to_json())

# convert the object into a dict
o_auth_token_response_dict = o_auth_token_response_instance.to_dict()
# create an instance of OAuthTokenResponse from a dict
o_auth_token_response_from_dict = OAuthTokenResponse.from_dict(o_auth_token_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


