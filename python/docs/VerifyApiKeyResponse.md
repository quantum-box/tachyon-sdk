# VerifyApiKeyResponse

Response for API key verification

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** |  | 
**service_account_id** | **str** |  | 
**tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.verify_api_key_response import VerifyApiKeyResponse

# TODO update the JSON string below
json = "{}"
# create an instance of VerifyApiKeyResponse from a JSON string
verify_api_key_response_instance = VerifyApiKeyResponse.from_json(json)
# print the JSON string representation of the object
print(VerifyApiKeyResponse.to_json())

# convert the object into a dict
verify_api_key_response_dict = verify_api_key_response_instance.to_dict()
# create an instance of VerifyApiKeyResponse from a dict
verify_api_key_response_from_dict = VerifyApiKeyResponse.from_dict(verify_api_key_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


