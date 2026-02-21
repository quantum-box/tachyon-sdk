# ProductVariantListResponse

Product variant list response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[ProductVariantResponse]**](ProductVariantResponse.md) | Variant items | 

## Example

```python
from tachyon_sdk.models.product_variant_list_response import ProductVariantListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ProductVariantListResponse from a JSON string
product_variant_list_response_instance = ProductVariantListResponse.from_json(json)
# print the JSON string representation of the object
print(ProductVariantListResponse.to_json())

# convert the object into a dict
product_variant_list_response_dict = product_variant_list_response_instance.to_dict()
# create an instance of ProductVariantListResponse from a dict
product_variant_list_response_from_dict = ProductVariantListResponse.from_dict(product_variant_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


