# OrderCheckoutApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**processCheckout**](OrderCheckoutApi.md#processcheckout) | **POST** /v1/order/checkout | Process payment checkout for a quote |



## processCheckout

> CheckoutResponse processCheckout(checkoutRequest)

Process payment checkout for a quote

### Example

```ts
import {
  Configuration,
  OrderCheckoutApi,
} from '@tachyon/sdk';
import type { ProcessCheckoutRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderCheckoutApi();

  const body = {
    // CheckoutRequest
    checkoutRequest: ...,
  } satisfies ProcessCheckoutRequest;

  try {
    const data = await api.processCheckout(body);
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
| **checkoutRequest** | [CheckoutRequest](CheckoutRequest.md) |  | |

### Return type

[**CheckoutResponse**](CheckoutResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Checkout session created |  -  |
| **400** | Bad request |  -  |
| **402** | Payment required |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

