# SignInWithPlatformRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**access_token** | **str** |  | 
**allow_sign_up** | **bool** |  | [optional] [default to True]
**email** | **str** |  | [optional] 
**name** | **str** |  | [optional] 
**platform_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.sign_in_with_platform_request import SignInWithPlatformRequest

# TODO update the JSON string below
json = "{}"
# create an instance of SignInWithPlatformRequest from a JSON string
sign_in_with_platform_request_instance = SignInWithPlatformRequest.from_json(json)
# print the JSON string representation of the object
print(SignInWithPlatformRequest.to_json())

# convert the object into a dict
sign_in_with_platform_request_dict = sign_in_with_platform_request_instance.to_dict()
# create an instance of SignInWithPlatformRequest from a dict
sign_in_with_platform_request_from_dict = SignInWithPlatformRequest.from_dict(sign_in_with_platform_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


