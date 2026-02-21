# tachyon_sdk.OrderRevenueApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**calculate_recurring_revenue**](OrderRevenueApi.md#calculate_recurring_revenue) | **POST** /v1/order/revenue/recurring | Calculate recurring revenue for a given period


# **calculate_recurring_revenue**
> RecurringRevenueResponse calculate_recurring_revenue(recurring_revenue_request)

Calculate recurring revenue for a given period

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.recurring_revenue_request import RecurringRevenueRequest
from tachyon_sdk.models.recurring_revenue_response import RecurringRevenueResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.OrderRevenueApi(api_client)
    recurring_revenue_request = tachyon_sdk.RecurringRevenueRequest() # RecurringRevenueRequest | 

    try:
        # Calculate recurring revenue for a given period
        api_response = api_instance.calculate_recurring_revenue(recurring_revenue_request)
        print("The response of OrderRevenueApi->calculate_recurring_revenue:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderRevenueApi->calculate_recurring_revenue: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **recurring_revenue_request** | [**RecurringRevenueRequest**](RecurringRevenueRequest.md)|  | 

### Return type

[**RecurringRevenueResponse**](RecurringRevenueResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Recurring revenue |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

