# OAuthTokenDetailResponse

Response for OAuth token detail

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**access_token** | **str** |  | 
**expires_at** | **str** |  | 
**provider** | **str** |  | 
**provider_user_id** | **str** |  | 
**refresh_token** | **str** |  | [optional] 

## Example

```python
from tachyon_sdk.models.o_auth_token_detail_response import OAuthTokenDetailResponse

# TODO update the JSON string below
json = "{}"
# create an instance of OAuthTokenDetailResponse from a JSON string
o_auth_token_detail_response_instance = OAuthTokenDetailResponse.from_json(json)
# print the JSON string representation of the object
print(OAuthTokenDetailResponse.to_json())

# convert the object into a dict
o_auth_token_detail_response_dict = o_auth_token_detail_response_instance.to_dict()
# create an instance of OAuthTokenDetailResponse from a dict
o_auth_token_detail_response_from_dict = OAuthTokenDetailResponse.from_dict(o_auth_token_detail_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


