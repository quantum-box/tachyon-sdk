# PurchaseOrderResponse

Purchase order response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**billing_info** | **str** | Billing information ID | [optional] 
**client_id** | **str** | Client ID | 
**currency** | **str** | Transaction currency | 
**delivery_date** | **str** | Delivery date | [optional] 
**id** | **str** | Purchase order ID | 
**invoice_address** | **str** | Invoice address ID | [optional] 
**line_items** | [**List[LineItemResponse]**](LineItemResponse.md) | Line items | 
**order_date** | **str** | Order date | 
**quotes_id** | **str** | Quote ID | 
**software_tenant_id** | **str** | Software tenant ID | [optional] 
**status** | **str** | Order status | 
**subtotal** | **float** | Subtotal | 
**tax** | **float** | Tax rate | 
**tenant_id** | **str** | Tenant ID | 
**total** | **float** | Total amount | 

## Example

```python
from tachyon_sdk.models.purchase_order_response import PurchaseOrderResponse

# TODO update the JSON string below
json = "{}"
# create an instance of PurchaseOrderResponse from a JSON string
purchase_order_response_instance = PurchaseOrderResponse.from_json(json)
# print the JSON string representation of the object
print(PurchaseOrderResponse.to_json())

# convert the object into a dict
purchase_order_response_dict = purchase_order_response_instance.to_dict()
# create an instance of PurchaseOrderResponse from a dict
purchase_order_response_from_dict = PurchaseOrderResponse.from_dict(purchase_order_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


