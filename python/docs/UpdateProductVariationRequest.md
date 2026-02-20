# UpdateProductVariationRequest

Update variation input

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**code** | **str** | Variation code | [optional] 
**currency** | **str** | Currency code | [optional] 
**id** | **str** | Variation ID | 
**metadata** | **object** |  | [optional] 
**name** | **str** | Variation name | [optional] 
**publication_description** | **str** | Publication description | [optional] 
**publication_name** | **str** | Publication name | [optional] 
**recurring** | **str** | Recurring billing frequency | [optional] 
**status** | **str** | Variation status | [optional] 
**unit_amount** | **int** | Unit amount | [optional] 

## Example

```python
from tachyon_sdk.models.update_product_variation_request import UpdateProductVariationRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateProductVariationRequest from a JSON string
update_product_variation_request_instance = UpdateProductVariationRequest.from_json(json)
# print the JSON string representation of the object
print(UpdateProductVariationRequest.to_json())

# convert the object into a dict
update_product_variation_request_dict = update_product_variation_request_instance.to_dict()
# create an instance of UpdateProductVariationRequest from a dict
update_product_variation_request_from_dict = UpdateProductVariationRequest.from_dict(update_product_variation_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


