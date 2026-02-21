# RecurringRevenueRequest

Request body for calculating recurring revenue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**end_date** | **str** | Period end date (YYYY-MM-DD) | 
**revenue_cycle** | **str** | Revenue cycle: WEEKLY or MONTHLY | 
**start_date** | **str** | Period start date (YYYY-MM-DD) | 

## Example

```python
from tachyon_sdk.models.recurring_revenue_request import RecurringRevenueRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RecurringRevenueRequest from a JSON string
recurring_revenue_request_instance = RecurringRevenueRequest.from_json(json)
# print the JSON string representation of the object
print(RecurringRevenueRequest.to_json())

# convert the object into a dict
recurring_revenue_request_dict = recurring_revenue_request_instance.to_dict()
# create an instance of RecurringRevenueRequest from a dict
recurring_revenue_request_from_dict = RecurringRevenueRequest.from_dict(recurring_revenue_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


