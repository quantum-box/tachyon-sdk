# ContractAuthorityEvidence

Opaque proof metadata returned for a current exact resolution.  The issuer and key identify the server-trusted verification record.  They do not expose private key material, party graphs or the signed token.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**evidence_digest** | **str** |  | 
**evidence_id** | **str** |  | 
**evidence_revision** | **str** |  | 
**issuer** | [**AuthorityIssuer**](AuthorityIssuer.md) |  | 
**revoked_at** | **datetime** |  | [optional] 
**valid_from** | **datetime** |  | 
**valid_until** | **datetime** |  | 

## Example

```python
from tachyon_sdk.models.contract_authority_evidence import ContractAuthorityEvidence

# TODO update the JSON string below
json = "{}"
# create an instance of ContractAuthorityEvidence from a JSON string
contract_authority_evidence_instance = ContractAuthorityEvidence.from_json(json)
# print the JSON string representation of the object
print(ContractAuthorityEvidence.to_json())

# convert the object into a dict
contract_authority_evidence_dict = contract_authority_evidence_instance.to_dict()
# create an instance of ContractAuthorityEvidence from a dict
contract_authority_evidence_from_dict = ContractAuthorityEvidence.from_dict(contract_authority_evidence_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


