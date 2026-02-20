# PurchaseOrderListResponse

Purchase order list response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[PurchaseOrderResponse]**](PurchaseOrderResponse.md) | Purchase order items | 

## Example

```python
from tachyon_sdk.models.purchase_order_list_response import PurchaseOrderListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of PurchaseOrderListResponse from a JSON string
purchase_order_list_response_instance = PurchaseOrderListResponse.from_json(json)
# print the JSON string representation of the object
print(PurchaseOrderListResponse.to_json())

# convert the object into a dict
purchase_order_list_response_dict = purchase_order_list_response_instance.to_dict()
# create an instance of PurchaseOrderListResponse from a dict
purchase_order_list_response_from_dict = PurchaseOrderListResponse.from_dict(purchase_order_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


