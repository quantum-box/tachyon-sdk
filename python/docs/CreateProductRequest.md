# CreateProductRequest

Request body for creating a product

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**billing_cycle** | **str** | Billing cycle: MONTHLY, YEARLY, ONE_TIME | 
**create_crm** | **bool** | Whether to sync with CRM | [optional] 
**description** | **str** | Product description | [optional] 
**id** | **str** | Optional product ID (auto-generated if omitted) | [optional] 
**image_file_ids** | **List[str]** | File IDs for product images | [optional] 
**jan_code** | **str** | JAN code | [optional] 
**kind** | **str** | Product type: PLAN or PRODUCT | 
**list_price** | **int** | List price in smallest currency unit | 
**name** | **str** | Product name | 
**publication_description** | **str** | Publication display description | [optional] 
**publication_name** | **str** | Publication display name | [optional] 
**publication_status** | **str** | Publication status | [optional] 
**sku_code** | **str** | SKU code | [optional] 
**status** | **str** | Product status: DRAFT, ACTIVE, ARCHIVED | 
**upc_code** | **str** | UPC code | [optional] 
**variations** | [**List[ProductVariationRequest]**](ProductVariationRequest.md) | Product variations | [optional] 

## Example

```python
from tachyon_sdk.models.create_product_request import CreateProductRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateProductRequest from a JSON string
create_product_request_instance = CreateProductRequest.from_json(json)
# print the JSON string representation of the object
print(CreateProductRequest.to_json())

# convert the object into a dict
create_product_request_dict = create_product_request_instance.to_dict()
# create an instance of CreateProductRequest from a dict
create_product_request_from_dict = CreateProductRequest.from_dict(create_product_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


