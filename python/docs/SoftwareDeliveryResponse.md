# SoftwareDeliveryResponse

Response for software delivery

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**access_url** | **str** |  | 
**created_at** | **str** |  | 
**delivered_at** | **str** |  | 
**id** | **str** |  | 
**order_id** | **str** |  | 
**status** | **str** |  | 
**updated_at** | **str** |  | 

## Example

```python
from tachyon_sdk.models.software_delivery_response import SoftwareDeliveryResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SoftwareDeliveryResponse from a JSON string
software_delivery_response_instance = SoftwareDeliveryResponse.from_json(json)
# print the JSON string representation of the object
print(SoftwareDeliveryResponse.to_json())

# convert the object into a dict
software_delivery_response_dict = software_delivery_response_instance.to_dict()
# create an instance of SoftwareDeliveryResponse from a dict
software_delivery_response_from_dict = SoftwareDeliveryResponse.from_dict(software_delivery_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


