# AddressRequest

Address input

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**address1** | **str** | Address line 1 | 
**address2** | **str** | Address line 2 | [optional] 
**city** | **str** | City | 
**postal_code** | **str** | Postal code | 
**state** | **str** | State/Prefecture | 

## Example

```python
from tachyon_sdk.models.address_request import AddressRequest

# TODO update the JSON string below
json = "{}"
# create an instance of AddressRequest from a JSON string
address_request_instance = AddressRequest.from_json(json)
# print the JSON string representation of the object
print(AddressRequest.to_json())

# convert the object into a dict
address_request_dict = address_request_instance.to_dict()
# create an instance of AddressRequest from a dict
address_request_from_dict = AddressRequest.from_dict(address_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


