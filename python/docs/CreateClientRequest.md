# CreateClientRequest

Request body for creating a client

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**address** | [**AddressRequest**](AddressRequest.md) | Head office address | [optional] 
**capital** | **int** | Capital | [optional] 
**corporate_number** | **str** | Corporate number | [optional] 
**create_crm** | **bool** | Whether to sync with CRM | [optional] 
**email** | **str** | Email address | [optional] 
**fax_number** | **str** | Fax number | [optional] 
**founded** | **str** | Establishment date | [optional] 
**id** | **str** | Optional client ID (auto-generated if omitted) | [optional] 
**industry** | **str** | Industry type | [optional] 
**listed** | **bool** | Listed or unlisted | [optional] 
**name** | **str** | Client name | 
**phone_number** | **str** | Phone number | [optional] 
**representative** | **str** | Representative name | [optional] 

## Example

```python
from tachyon_sdk.models.create_client_request import CreateClientRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateClientRequest from a JSON string
create_client_request_instance = CreateClientRequest.from_json(json)
# print the JSON string representation of the object
print(CreateClientRequest.to_json())

# convert the object into a dict
create_client_request_dict = create_client_request_instance.to_dict()
# create an instance of CreateClientRequest from a dict
create_client_request_from_dict = CreateClientRequest.from_dict(create_client_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


