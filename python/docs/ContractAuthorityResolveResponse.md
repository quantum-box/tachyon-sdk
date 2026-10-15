# ContractAuthorityResolveResponse

Current proof returned for the exact request.  The request is echoed so a consumer can compare every scope field before converting to an internal Field proof.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**evidence** | [**ContractAuthorityEvidence**](ContractAuthorityEvidence.md) |  | 
**request** | [**ContractAuthorityResolveRequest**](ContractAuthorityResolveRequest.md) |  | 
**schema_version** | **str** |  | 

## Example

```python
from tachyon_sdk.models.contract_authority_resolve_response import ContractAuthorityResolveResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ContractAuthorityResolveResponse from a JSON string
contract_authority_resolve_response_instance = ContractAuthorityResolveResponse.from_json(json)
# print the JSON string representation of the object
print(ContractAuthorityResolveResponse.to_json())

# convert the object into a dict
contract_authority_resolve_response_dict = contract_authority_resolve_response_instance.to_dict()
# create an instance of ContractAuthorityResolveResponse from a dict
contract_authority_resolve_response_from_dict = ContractAuthorityResolveResponse.from_dict(contract_authority_resolve_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


