# ProductListResponse

Paginated product list response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**has_next_page** | **bool** | Whether there is a next page | 
**items** | [**List[ProductResponse]**](ProductResponse.md) | Product items | 
**limit** | **int** | Page limit | 
**offset** | **int** | Page offset | 
**total_count** | **int** | Total number of products | 

## Example

```python
from tachyon_sdk.models.product_list_response import ProductListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ProductListResponse from a JSON string
product_list_response_instance = ProductListResponse.from_json(json)
# print the JSON string representation of the object
print(ProductListResponse.to_json())

# convert the object into a dict
product_list_response_dict = product_list_response_instance.to_dict()
# create an instance of ProductListResponse from a dict
product_list_response_from_dict = ProductListResponse.from_dict(product_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


