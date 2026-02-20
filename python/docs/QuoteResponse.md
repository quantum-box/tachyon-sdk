# QuoteResponse

Quote response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**client_id** | **str** | Client ID | [optional] 
**created_at** | **datetime** | Created at | 
**currency** | **str** | Transaction currency | 
**id** | **str** | Quote ID | 
**line_items** | [**List[LineItemResponse]**](LineItemResponse.md) | Line items | 
**order_date** | **str** | Order date | 
**software_tenant_id** | **str** | Software tenant ID | [optional] 
**status** | **str** | Quote status | 
**subtotal** | **float** | Subtotal | 
**tax** | **int** | Tax percentage | 
**tenant_id** | **str** | Tenant ID | 
**title** | **str** | Quote title | 
**total** | **float** | Total amount | 
**updated_at** | **datetime** | Updated at | 
**url** | **str** | Quote URL | [optional] 

## Example

```python
from tachyon_sdk.models.quote_response import QuoteResponse

# TODO update the JSON string below
json = "{}"
# create an instance of QuoteResponse from a JSON string
quote_response_instance = QuoteResponse.from_json(json)
# print the JSON string representation of the object
print(QuoteResponse.to_json())

# convert the object into a dict
quote_response_dict = quote_response_instance.to_dict()
# create an instance of QuoteResponse from a dict
quote_response_from_dict = QuoteResponse.from_dict(quote_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


