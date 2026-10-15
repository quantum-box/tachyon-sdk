# AuthorityIssuer

Issuer and signing-key identity returned by the trusted source.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**issuer_id** | **str** |  | 
**key_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.authority_issuer import AuthorityIssuer

# TODO update the JSON string below
json = "{}"
# create an instance of AuthorityIssuer from a JSON string
authority_issuer_instance = AuthorityIssuer.from_json(json)
# print the JSON string representation of the object
print(AuthorityIssuer.to_json())

# convert the object into a dict
authority_issuer_dict = authority_issuer_instance.to_dict()
# create an instance of AuthorityIssuer from a dict
authority_issuer_from_dict = AuthorityIssuer.from_dict(authority_issuer_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


