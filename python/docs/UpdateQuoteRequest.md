# UpdateQuoteRequest

Request body for updating a quote

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**billing_information_id** | **str** | Billing information ID | [optional] 
**line_items** | [**List[LineItemRequest]**](LineItemRequest.md) | Updated line items | [optional] 
**status** | **str** | Updated quote status | [optional] 
**title** | **str** | Updated quote title | [optional] 

## Example

```python
from tachyon_sdk.models.update_quote_request import UpdateQuoteRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateQuoteRequest from a JSON string
update_quote_request_instance = UpdateQuoteRequest.from_json(json)
# print the JSON string representation of the object
print(UpdateQuoteRequest.to_json())

# convert the object into a dict
update_quote_request_dict = update_quote_request_instance.to_dict()
# create an instance of UpdateQuoteRequest from a dict
update_quote_request_from_dict = UpdateQuoteRequest.from_dict(update_quote_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


