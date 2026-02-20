# OrderShippingApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**registerShippingDestination**](OrderShippingApi.md#registershippingdestinationoperation) | **POST** /v1/order/shipping-destinations | Register a shipping destination for a quote |



## registerShippingDestination

> ShippingDestinationResponse registerShippingDestination(registerShippingDestinationRequest)

Register a shipping destination for a quote

### Example

```ts
import {
  Configuration,
  OrderShippingApi,
} from '@tachyon/sdk';
import type { RegisterShippingDestinationOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderShippingApi();

  const body = {
    // RegisterShippingDestinationRequest
    registerShippingDestinationRequest: ...,
  } satisfies RegisterShippingDestinationOperationRequest;

  try {
    const data = await api.registerShippingDestination(body);
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
| **registerShippingDestinationRequest** | [RegisterShippingDestinationRequest](RegisterShippingDestinationRequest.md) |  | |

### Return type

[**ShippingDestinationResponse**](ShippingDestinationResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Shipping destination registered |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

