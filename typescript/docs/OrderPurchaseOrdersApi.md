# OrderPurchaseOrdersApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getPurchaseOrder**](OrderPurchaseOrdersApi.md#getpurchaseorder) | **GET** /v1/order/purchase-orders/{id} | Get a purchase order by ID |
| [**listPurchaseOrders**](OrderPurchaseOrdersApi.md#listpurchaseorders) | **GET** /v1/order/purchase-orders | List all purchase orders |



## getPurchaseOrder

> PurchaseOrderResponse getPurchaseOrder(id)

Get a purchase order by ID

### Example

```ts
import {
  Configuration,
  OrderPurchaseOrdersApi,
} from '@tachyon/sdk';
import type { GetPurchaseOrderRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderPurchaseOrdersApi();

  const body = {
    // string | Purchase order ID
    id: id_example,
  } satisfies GetPurchaseOrderRequest;

  try {
    const data = await api.getPurchaseOrder(body);
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
| **id** | `string` | Purchase order ID | [Defaults to `undefined`] |

### Return type

[**PurchaseOrderResponse**](PurchaseOrderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Purchase order found |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listPurchaseOrders

> PurchaseOrderListResponse listPurchaseOrders()

List all purchase orders

### Example

```ts
import {
  Configuration,
  OrderPurchaseOrdersApi,
} from '@tachyon/sdk';
import type { ListPurchaseOrdersRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderPurchaseOrdersApi();

  try {
    const data = await api.listPurchaseOrders();
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**PurchaseOrderListResponse**](PurchaseOrderListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Purchase order list |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

