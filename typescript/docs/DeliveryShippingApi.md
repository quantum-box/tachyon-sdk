# DeliveryShippingApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**checkShippingAvailability**](DeliveryShippingApi.md#checkshippingavailability) | **GET** /v1/delivery/shipping-destinations/{id}/availability | Check physical shipping availability |
| [**createShippingDestination**](DeliveryShippingApi.md#createshippingdestinationoperation) | **POST** /v1/delivery/shipping-destinations | Create a shipping destination |



## checkShippingAvailability

> ShippingAvailabilityResponse checkShippingAvailability(id)

Check physical shipping availability

### Example

```ts
import {
  Configuration,
  DeliveryShippingApi,
} from '@tachyon/sdk';
import type { CheckShippingAvailabilityRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new DeliveryShippingApi();

  const body = {
    // string | Shipping destination ID
    id: id_example,
  } satisfies CheckShippingAvailabilityRequest;

  try {
    const data = await api.checkShippingAvailability(body);
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
| **id** | `string` | Shipping destination ID | [Defaults to `undefined`] |

### Return type

[**ShippingAvailabilityResponse**](ShippingAvailabilityResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Availability check result |  -  |
| **400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createShippingDestination

> ShippingDestinationResponse createShippingDestination(createShippingDestinationRequest)

Create a shipping destination

### Example

```ts
import {
  Configuration,
  DeliveryShippingApi,
} from '@tachyon/sdk';
import type { CreateShippingDestinationOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new DeliveryShippingApi();

  const body = {
    // CreateShippingDestinationRequest
    createShippingDestinationRequest: ...,
  } satisfies CreateShippingDestinationOperationRequest;

  try {
    const data = await api.createShippingDestination(body);
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
| **createShippingDestinationRequest** | [CreateShippingDestinationRequest](CreateShippingDestinationRequest.md) |  | |

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
| **201** | Shipping destination created |  -  |
| **400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

