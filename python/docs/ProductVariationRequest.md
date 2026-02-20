# ProductVariationRequest

Product variation input

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**code** | **str** | Variation code | [optional] 
**currency** | **str** | Currency code (e.g. \&quot;JPY\&quot;) | 
**metadata** | **object** |  | [optional] 
**name** | **str** | Variation name | 
**publication_description** | **str** | Publication description | [optional] 
**publication_name** | **str** | Publication name | [optional] 
**recurring** | **str** | Recurring billing frequency | [optional] 
**status** | **str** | Variation status | [optional] 
**unit_amount** | **int** | Unit amount | 

## Example

```python
from tachyon_sdk.models.product_variation_request import ProductVariationRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ProductVariationRequest from a JSON string
product_variation_request_instance = ProductVariationRequest.from_json(json)
# print the JSON string representation of the object
print(ProductVariationRequest.to_json())

# convert the object into a dict
product_variation_request_dict = product_variation_request_instance.to_dict()
# create an instance of ProductVariationRequest from a dict
product_variation_request_from_dict = ProductVariationRequest.from_dict(product_variation_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


