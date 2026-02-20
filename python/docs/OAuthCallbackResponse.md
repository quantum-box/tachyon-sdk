# OAuthCallbackResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operator_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.o_auth_callback_response import OAuthCallbackResponse

# TODO update the JSON string below
json = "{}"
# create an instance of OAuthCallbackResponse from a JSON string
o_auth_callback_response_instance = OAuthCallbackResponse.from_json(json)
# print the JSON string representation of the object
print(OAuthCallbackResponse.to_json())

# convert the object into a dict
o_auth_callback_response_dict = o_auth_callback_response_instance.to_dict()
# create an instance of OAuthCallbackResponse from a dict
o_auth_callback_response_from_dict = OAuthCallbackResponse.from_dict(o_auth_callback_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


