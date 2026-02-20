# CreateQuoteRequest

Request body for creating a quote

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**client_id** | **str** | Client ID (optional) | [optional] 
**line_items** | [**List[LineItemRequest]**](LineItemRequest.md) | Line items | 
**title** | **str** | Quote title | 

## Example

```python
from tachyon_sdk.models.create_quote_request import CreateQuoteRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateQuoteRequest from a JSON string
create_quote_request_instance = CreateQuoteRequest.from_json(json)
# print the JSON string representation of the object
print(CreateQuoteRequest.to_json())

# convert the object into a dict
create_quote_request_dict = create_quote_request_instance.to_dict()
# create an instance of CreateQuoteRequest from a dict
create_quote_request_from_dict = CreateQuoteRequest.from_dict(create_quote_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


