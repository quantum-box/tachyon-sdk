# RegisterShippingDestinationRequest

Request body for registering a shipping destination

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**address** | [**AddressRequest**](AddressRequest.md) | Shipping address | [optional] 
**corporate_name** | **str** | Corporate name | [optional] 
**department_name** | **str** | Department name | [optional] 
**email** | **str** | Email address | 
**first_name** | **str** | First name | 
**last_name** | **str** | Last name | 
**phone_number** | **str** | Phone number | [optional] 
**position_name** | **str** | Position name | [optional] 
**quote_id** | **str** | Quote ID to associate with | 

## Example

```python
from tachyon_sdk.models.register_shipping_destination_request import RegisterShippingDestinationRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RegisterShippingDestinationRequest from a JSON string
register_shipping_destination_request_instance = RegisterShippingDestinationRequest.from_json(json)
# print the JSON string representation of the object
print(RegisterShippingDestinationRequest.to_json())

# convert the object into a dict
register_shipping_destination_request_dict = register_shipping_destination_request_instance.to_dict()
# create an instance of RegisterShippingDestinationRequest from a dict
register_shipping_destination_request_from_dict = RegisterShippingDestinationRequest.from_dict(register_shipping_destination_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


