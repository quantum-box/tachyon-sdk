# UpdateProductRequest

Request body for updating a product

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**billing_cycle** | **str** | Billing cycle | [optional] 
**description** | **str** | Product description | [optional] 
**image_file_ids** | **List[str]** | Updated image file IDs | [optional] 
**jan_code** | **str** | JAN code | [optional] 
**kind** | **str** | Product type | [optional] 
**list_price** | **int** | List price | [optional] 
**name** | **str** | Product name | [optional] 
**publication_description** | **str** | Publication description | [optional] 
**publication_name** | **str** | Publication name | [optional] 
**publication_status** | **str** | Publication status | [optional] 
**sku_code** | **str** | SKU code | [optional] 
**status** | **str** | Product status | [optional] 
**upc_code** | **str** | UPC code | [optional] 
**variations** | [**List[UpdateProductVariationRequest]**](UpdateProductVariationRequest.md) | Updated variations | [optional] 

## Example

```python
from tachyon_sdk.models.update_product_request import UpdateProductRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateProductRequest from a JSON string
update_product_request_instance = UpdateProductRequest.from_json(json)
# print the JSON string representation of the object
print(UpdateProductRequest.to_json())

# convert the object into a dict
update_product_request_dict = update_product_request_instance.to_dict()
# create an instance of UpdateProductRequest from a dict
update_product_request_from_dict = UpdateProductRequest.from_dict(update_product_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


