# IssueQuoteRequest

Request body for issuing a quote

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**client_id** | **str** | Client ID to issue the quote to | 

## Example

```python
from tachyon_sdk.models.issue_quote_request import IssueQuoteRequest

# TODO update the JSON string below
json = "{}"
# create an instance of IssueQuoteRequest from a JSON string
issue_quote_request_instance = IssueQuoteRequest.from_json(json)
# print the JSON string representation of the object
print(IssueQuoteRequest.to_json())

# convert the object into a dict
issue_quote_request_dict = issue_quote_request_instance.to_dict()
# create an instance of IssueQuoteRequest from a dict
issue_quote_request_from_dict = IssueQuoteRequest.from_dict(issue_quote_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


