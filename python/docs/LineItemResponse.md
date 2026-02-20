# LineItemResponse

Line item response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**discount** | **float** | Discount | 
**id** | **str** | Line item ID | 
**name** | **str** | Line item name | 
**product_id** | **str** | Product ID | 
**quantity** | **int** | Quantity | 
**unit_price** | **float** | Unit price | 

## Example

```python
from tachyon_sdk.models.line_item_response import LineItemResponse

# TODO update the JSON string below
json = "{}"
# create an instance of LineItemResponse from a JSON string
line_item_response_instance = LineItemResponse.from_json(json)
# print the JSON string representation of the object
print(LineItemResponse.to_json())

# convert the object into a dict
line_item_response_dict = line_item_response_instance.to_dict()
# create an instance of LineItemResponse from a dict
line_item_response_from_dict = LineItemResponse.from_dict(line_item_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


