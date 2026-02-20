# ShippingAvailabilityResponse

Response for shipping availability check

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**available** | **bool** |  | 

## Example

```python
from tachyon_sdk.models.shipping_availability_response import ShippingAvailabilityResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ShippingAvailabilityResponse from a JSON string
shipping_availability_response_instance = ShippingAvailabilityResponse.from_json(json)
# print the JSON string representation of the object
print(ShippingAvailabilityResponse.to_json())

# convert the object into a dict
shipping_availability_response_dict = shipping_availability_response_instance.to_dict()
# create an instance of ShippingAvailabilityResponse from a dict
shipping_availability_response_from_dict = ShippingAvailabilityResponse.from_dict(shipping_availability_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


