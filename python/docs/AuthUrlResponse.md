# AuthUrlResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | **str** | OAuth authorization URL | 

## Example

```python
from tachyon_sdk.models.auth_url_response import AuthUrlResponse

# TODO update the JSON string below
json = "{}"
# create an instance of AuthUrlResponse from a JSON string
auth_url_response_instance = AuthUrlResponse.from_json(json)
# print the JSON string representation of the object
print(AuthUrlResponse.to_json())

# convert the object into a dict
auth_url_response_dict = auth_url_response_instance.to_dict()
# create an instance of AuthUrlResponse from a dict
auth_url_response_from_dict = AuthUrlResponse.from_dict(auth_url_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


