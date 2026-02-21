# RecurringRevenueResponse

Recurring revenue response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**amount** | **float** | Revenue amount | 
**change_percentage** | **float** | Change percentage from previous period | [optional] 
**created_at** | **datetime** | Created at | 
**cycle** | **str** | Revenue cycle | 
**end_date** | **str** | Period end date | 
**id** | **str** | Revenue ID | 
**start_date** | **str** | Period start date | 
**tenant_id** | **str** | Tenant ID | 

## Example

```python
from tachyon_sdk.models.recurring_revenue_response import RecurringRevenueResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RecurringRevenueResponse from a JSON string
recurring_revenue_response_instance = RecurringRevenueResponse.from_json(json)
# print the JSON string representation of the object
print(RecurringRevenueResponse.to_json())

# convert the object into a dict
recurring_revenue_response_dict = recurring_revenue_response_instance.to_dict()
# create an instance of RecurringRevenueResponse from a dict
recurring_revenue_response_from_dict = RecurringRevenueResponse.from_dict(recurring_revenue_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


