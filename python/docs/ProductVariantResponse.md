# ProductVariantResponse

Product variant response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**code** | **str** | Variant code | 
**created_at** | **datetime** | Created at | 
**id** | **str** | Variant ID | 
**metadata** | **object** |  | 
**name** | **str** | Variant name | 
**product_id** | **str** | Product ID | 
**status** | **str** | Variant status | 
**tenant_id** | **str** | Tenant ID | 
**updated_at** | **datetime** | Updated at | 

## Example

```python
from tachyon_sdk.models.product_variant_response import ProductVariantResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ProductVariantResponse from a JSON string
product_variant_response_instance = ProductVariantResponse.from_json(json)
# print the JSON string representation of the object
print(ProductVariantResponse.to_json())

# convert the object into a dict
product_variant_response_dict = product_variant_response_instance.to_dict()
# create an instance of ProductVariantResponse from a dict
product_variant_response_from_dict = ProductVariantResponse.from_dict(product_variant_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


