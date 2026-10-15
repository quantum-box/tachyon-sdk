# BindingReference

Exact Field contract identity and binding version/digest.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**binding_digest** | **str** |  | 
**binding_revision** | **str** |  | 
**contract_id** | **str** |  | 
**record_owner_tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.binding_reference import BindingReference

# TODO update the JSON string below
json = "{}"
# create an instance of BindingReference from a JSON string
binding_reference_instance = BindingReference.from_json(json)
# print the JSON string representation of the object
print(BindingReference.to_json())

# convert the object into a dict
binding_reference_dict = binding_reference_instance.to_dict()
# create an instance of BindingReference from a dict
binding_reference_from_dict = BindingReference.from_dict(binding_reference_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


