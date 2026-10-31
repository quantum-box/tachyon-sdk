# OAuth2UpdateClientRequest

Request to update an OAuth2 client

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allowed_scopes** | **List[str]** | Allowed scopes | [optional] 
**name** | **str** | Display name | [optional] 
**redirect_uris** | **List[str]** | Allowed redirect URIs | [optional] 
**status** | **str** | Status: \&quot;active\&quot; or \&quot;inactive\&quot; | [optional] 

## Example

```python
from tachyon_sdk.models.o_auth2_update_client_request import OAuth2UpdateClientRequest

# TODO update the JSON string below
json = "{}"
# create an instance of OAuth2UpdateClientRequest from a JSON string
o_auth2_update_client_request_instance = OAuth2UpdateClientRequest.from_json(json)
# print the JSON string representation of the object
print(OAuth2UpdateClientRequest.to_json())

# convert the object into a dict
o_auth2_update_client_request_dict = o_auth2_update_client_request_instance.to_dict()
# create an instance of OAuth2UpdateClientRequest from a dict
o_auth2_update_client_request_from_dict = OAuth2UpdateClientRequest.from_dict(o_auth2_update_client_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


