# ClientResponse

Client response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**capital** | **int** | Capital | [optional] 
**corporate_number** | **str** | Corporate number | [optional] 
**email** | **str** | Email address | [optional] 
**fax_number** | **str** | Fax number | [optional] 
**founded** | **str** | Establishment date | [optional] 
**head_office_address** | [**AddressResponse**](AddressResponse.md) | Head office address | [optional] 
**id** | **str** | Client ID | 
**industry** | **str** | Industry type | [optional] 
**listed** | **bool** | Listed or unlisted | [optional] 
**name** | **str** | Client name | 
**phone_number** | **str** | Phone number | [optional] 
**representative** | **str** | Representative | [optional] 
**tenant_id** | **str** | Tenant ID | 

## Example

```python
from tachyon_sdk.models.client_response import ClientResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ClientResponse from a JSON string
client_response_instance = ClientResponse.from_json(json)
# print the JSON string representation of the object
print(ClientResponse.to_json())

# convert the object into a dict
client_response_dict = client_response_instance.to_dict()
# create an instance of ClientResponse from a dict
client_response_from_dict = ClientResponse.from_dict(client_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


