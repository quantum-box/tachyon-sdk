# OrderRevenueApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**calculateRecurringRevenue**](OrderRevenueApi.md#calculaterecurringrevenue) | **POST** /v1/order/revenue/recurring | Calculate recurring revenue for a given period |



## calculateRecurringRevenue

> RecurringRevenueResponse calculateRecurringRevenue(recurringRevenueRequest)

Calculate recurring revenue for a given period

### Example

```ts
import {
  Configuration,
  OrderRevenueApi,
} from '@tachyon/sdk';
import type { CalculateRecurringRevenueRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderRevenueApi();

  const body = {
    // RecurringRevenueRequest
    recurringRevenueRequest: ...,
  } satisfies CalculateRecurringRevenueRequest;

  try {
    const data = await api.calculateRecurringRevenue(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **recurringRevenueRequest** | [RecurringRevenueRequest](RecurringRevenueRequest.md) |  | |

### Return type

[**RecurringRevenueResponse**](RecurringRevenueResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Recurring revenue |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

