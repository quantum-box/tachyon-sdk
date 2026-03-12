# UpdateClientRequest

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
from tachyon_sdk.models.update_client_request import UpdateClientRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateClientRequest from a JSON string
update_client_request_instance = UpdateClientRequest.from_json(json)
# print the JSON string representation of the object
print(UpdateClientRequest.to_json())

# convert the object into a dict
update_client_request_dict = update_client_request_instance.to_dict()
# create an instance of UpdateClientRequest from a dict
update_client_request_from_dict = UpdateClientRequest.from_dict(update_client_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


