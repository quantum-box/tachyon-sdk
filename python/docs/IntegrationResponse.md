# IntegrationResponse


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

## Example

```python
from tachyon_sdk.models.integration_response import IntegrationResponse

# TODO update the JSON string below
json = "{}"
# create an instance of IntegrationResponse from a JSON string
integration_response_instance = IntegrationResponse.from_json(json)
# print the JSON string representation of the object
print(IntegrationResponse.to_json())

# convert the object into a dict
integration_response_dict = integration_response_instance.to_dict()
# create an instance of IntegrationResponse from a dict
integration_response_from_dict = IntegrationResponse.from_dict(integration_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


