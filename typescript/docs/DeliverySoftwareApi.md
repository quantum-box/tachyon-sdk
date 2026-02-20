# DeliverySoftwareApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getSoftwareDeliveryByOrder**](DeliverySoftwareApi.md#getsoftwaredeliverybyorder) | **GET** /v1/delivery/software/by-order/{order_id} | Get software delivery by order ID |



## getSoftwareDeliveryByOrder

> SoftwareDeliveryResponse getSoftwareDeliveryByOrder(orderId)

Get software delivery by order ID

### Example

```ts
import {
  Configuration,
  DeliverySoftwareApi,
} from '@tachyon/sdk';
import type { GetSoftwareDeliveryByOrderRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new DeliverySoftwareApi();

  const body = {
    // string | Purchase order ID
    orderId: orderId_example,
  } satisfies GetSoftwareDeliveryByOrderRequest;

  try {
    const data = await api.getSoftwareDeliveryByOrder(body);
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
| **orderId** | `string` | Purchase order ID | [Defaults to `undefined`] |

### Return type

[**SoftwareDeliveryResponse**](SoftwareDeliveryResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Software delivery found |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

