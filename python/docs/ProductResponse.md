# ProductResponse

Product response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**billing_cycle** | **str** | Billing cycle | 
**created_at** | **datetime** | Created at | 
**description** | **str** | Product description | [optional] 
**id** | **str** | Product ID | 
**image_file_ids** | **List[str]** | Image file IDs | 
**jan_code** | **str** | JAN code | [optional] 
**kind** | **str** | Product type | 
**list_price** | **int** | List price | 
**name** | **str** | Product name | 
**publication_description** | **str** | Publication description | [optional] 
**publication_name** | **str** | Publication name | [optional] 
**publication_status** | **str** | Publication status | 
**sku_code** | **str** | SKU code | [optional] 
**status** | **str** | Product status | 
**tenant_id** | **str** | Tenant ID | 
**upc_code** | **str** | UPC code | [optional] 
**updated_at** | **datetime** | Updated at | 

## Example

```python
from tachyon_sdk.models.product_response import ProductResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ProductResponse from a JSON string
product_response_instance = ProductResponse.from_json(json)
# print the JSON string representation of the object
print(ProductResponse.to_json())

# convert the object into a dict
product_response_dict = product_response_instance.to_dict()
# create an instance of ProductResponse from a dict
product_response_from_dict = ProductResponse.from_dict(product_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


