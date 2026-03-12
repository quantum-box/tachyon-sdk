# IntegrationDetailResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**category** | **str** |  | 
**description** | **str** |  | 
**icon_url** | **str** |  | [optional] 
**id** | **str** |  | 
**is_enabled** | **bool** |  | 
**is_featured** | **bool** |  | 
**name** | **str** |  | 
**provider** | **str** |  | 
**requires_oauth** | **bool** |  | 
**supported_objects** | **List[str]** |  | 
**sync_capability** | **str** |  | 

## Example

```python
from tachyon_sdk.models.integration_detail_response import IntegrationDetailResponse

# TODO update the JSON string below
json = "{}"
# create an instance of IntegrationDetailResponse from a JSON string
integration_detail_response_instance = IntegrationDetailResponse.from_json(json)
# print the JSON string representation of the object
print(IntegrationDetailResponse.to_json())

# convert the object into a dict
integration_detail_response_dict = integration_detail_response_instance.to_dict()
# create an instance of IntegrationDetailResponse from a dict
integration_detail_response_from_dict = IntegrationDetailResponse.from_dict(integration_detail_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


