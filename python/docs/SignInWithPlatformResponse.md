# SignInWithPlatformResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user** | [**User**](User.md) |  | 

## Example

```python
from tachyon_sdk.models.sign_in_with_platform_response import SignInWithPlatformResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SignInWithPlatformResponse from a JSON string
sign_in_with_platform_response_instance = SignInWithPlatformResponse.from_json(json)
# print the JSON string representation of the object
print(SignInWithPlatformResponse.to_json())

# convert the object into a dict
sign_in_with_platform_response_dict = sign_in_with_platform_response_instance.to_dict()
# create an instance of SignInWithPlatformResponse from a dict
sign_in_with_platform_response_from_dict = SignInWithPlatformResponse.from_dict(sign_in_with_platform_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


