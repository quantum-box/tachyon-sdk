# VariantLinkResponse

Variant procurement link response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **str** | Link ID | 
**metadata** | **object** |  | 
**procurement_code** | **str** | Procurement code | 
**supplier_id** | **str** | Supplier ID | 
**tenant_id** | **str** | Tenant ID | 
**variant_id** | **str** | Product variant ID | 

## Example

```python
from tachyon_sdk.models.variant_link_response import VariantLinkResponse

# TODO update the JSON string below
json = "{}"
# create an instance of VariantLinkResponse from a JSON string
variant_link_response_instance = VariantLinkResponse.from_json(json)
# print the JSON string representation of the object
print(VariantLinkResponse.to_json())

# convert the object into a dict
variant_link_response_dict = variant_link_response_instance.to_dict()
# create an instance of VariantLinkResponse from a dict
variant_link_response_from_dict = VariantLinkResponse.from_dict(variant_link_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


