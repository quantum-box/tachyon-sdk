# ContractAuthorityResolveRequest

The current authority request sent by Field.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | **str** |  | 
**actor** | [**AuthorityActor**](AuthorityActor.md) |  | 
**audience** | **str** |  | 
**binding** | [**BindingReference**](BindingReference.md) |  | 
**environment** | **str** |  | 
**resource** | **str** |  | 
**schema_version** | **str** |  | 
**slot** | [**BindingSlot**](BindingSlot.md) |  | 

## Example

```python
from tachyon_sdk.models.contract_authority_resolve_request import ContractAuthorityResolveRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ContractAuthorityResolveRequest from a JSON string
contract_authority_resolve_request_instance = ContractAuthorityResolveRequest.from_json(json)
# print the JSON string representation of the object
print(ContractAuthorityResolveRequest.to_json())

# convert the object into a dict
contract_authority_resolve_request_dict = contract_authority_resolve_request_instance.to_dict()
# create an instance of ContractAuthorityResolveRequest from a dict
contract_authority_resolve_request_from_dict = ContractAuthorityResolveRequest.from_dict(contract_authority_resolve_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


