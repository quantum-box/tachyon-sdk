# ShippingDestinationResponse

Shipping destination response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **str** | Shipping destination ID | 

## Example

```python
from tachyon_sdk.models.shipping_destination_response import ShippingDestinationResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ShippingDestinationResponse from a JSON string
shipping_destination_response_instance = ShippingDestinationResponse.from_json(json)
# print the JSON string representation of the object
print(ShippingDestinationResponse.to_json())

# convert the object into a dict
shipping_destination_response_dict = shipping_destination_response_instance.to_dict()
# create an instance of ShippingDestinationResponse from a dict
shipping_destination_response_from_dict = ShippingDestinationResponse.from_dict(shipping_destination_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


