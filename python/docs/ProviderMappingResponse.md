# ProviderMappingResponse

Provider mapping response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**provider_id** | **str** | Provider&#39;s external ID | 
**provider_name** | **str** | Provider name (e.g. \&quot;stripe\&quot;) | 

## Example

```python
from tachyon_sdk.models.provider_mapping_response import ProviderMappingResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ProviderMappingResponse from a JSON string
provider_mapping_response_instance = ProviderMappingResponse.from_json(json)
# print the JSON string representation of the object
print(ProviderMappingResponse.to_json())

# convert the object into a dict
provider_mapping_response_dict = provider_mapping_response_instance.to_dict()
# create an instance of ProviderMappingResponse from a dict
provider_mapping_response_from_dict = ProviderMappingResponse.from_dict(provider_mapping_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


