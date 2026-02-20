# LineItemRequest

Line item input

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**discount** | **float** | Discount | [optional] 
**id** | **str** | Line item ID (optional, for updates) | [optional] 
**name** | **str** | Line item name | [optional] 
**product_id** | **str** | Product ID | 
**quantity** | **int** | Quantity | 
**unit_price** | **float** | Unit price | 

## Example

```python
from tachyon_sdk.models.line_item_request import LineItemRequest

# TODO update the JSON string below
json = "{}"
# create an instance of LineItemRequest from a JSON string
line_item_request_instance = LineItemRequest.from_json(json)
# print the JSON string representation of the object
print(LineItemRequest.to_json())

# convert the object into a dict
line_item_request_dict = line_item_request_instance.to_dict()
# create an instance of LineItemRequest from a dict
line_item_request_from_dict = LineItemRequest.from_dict(line_item_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


