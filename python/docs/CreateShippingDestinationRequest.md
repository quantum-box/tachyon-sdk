# CreateShippingDestinationRequest

Request to create a shipping destination

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**address1** | **str** | Address line 1 | [optional] 
**address2** | **str** | Address line 2 | [optional] 
**city** | **str** | City | [optional] 
**corporate_name** | **str** |  | [optional] 
**department_name** | **str** |  | [optional] 
**email** | **str** |  | 
**first_name** | **str** |  | 
**last_name** | **str** |  | 
**phone_number** | **str** |  | [optional] 
**position_name** | **str** |  | [optional] 
**postal_code** | **str** | Postal code | [optional] 
**state** | **str** | State/prefecture | [optional] 

## Example

```python
from tachyon_sdk.models.create_shipping_destination_request import CreateShippingDestinationRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateShippingDestinationRequest from a JSON string
create_shipping_destination_request_instance = CreateShippingDestinationRequest.from_json(json)
# print the JSON string representation of the object
print(CreateShippingDestinationRequest.to_json())

# convert the object into a dict
create_shipping_destination_request_dict = create_shipping_destination_request_instance.to_dict()
# create an instance of CreateShippingDestinationRequest from a dict
create_shipping_destination_request_from_dict = CreateShippingDestinationRequest.from_dict(create_shipping_destination_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


