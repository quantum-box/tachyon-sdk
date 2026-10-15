# AuthorityPartyRef

A legal-party reference from the stored Field contract.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**party_id** | **str** |  | 
**record_owner_tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.authority_party_ref import AuthorityPartyRef

# TODO update the JSON string below
json = "{}"
# create an instance of AuthorityPartyRef from a JSON string
authority_party_ref_instance = AuthorityPartyRef.from_json(json)
# print the JSON string representation of the object
print(AuthorityPartyRef.to_json())

# convert the object into a dict
authority_party_ref_dict = authority_party_ref_instance.to_dict()
# create an instance of AuthorityPartyRef from a dict
authority_party_ref_from_dict = AuthorityPartyRef.from_dict(authority_party_ref_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


