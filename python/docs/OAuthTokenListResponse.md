# OAuthTokenListResponse

Response for OAuth token list

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tokens** | [**List[OAuthTokenResponse]**](OAuthTokenResponse.md) |  | 

## Example

```python
from tachyon_sdk.models.o_auth_token_list_response import OAuthTokenListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of OAuthTokenListResponse from a JSON string
o_auth_token_list_response_instance = OAuthTokenListResponse.from_json(json)
# print the JSON string representation of the object
print(OAuthTokenListResponse.to_json())

# convert the object into a dict
o_auth_token_list_response_dict = o_auth_token_list_response_instance.to_dict()
# create an instance of OAuthTokenListResponse from a dict
o_auth_token_list_response_from_dict = OAuthTokenListResponse.from_dict(o_auth_token_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


