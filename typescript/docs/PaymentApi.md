# PaymentApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getOrCreateStripeCustomer**](PaymentApi.md#getorcreatestripecustomer) | **POST** /v1/payment/stripe-customer | Get or create a Stripe customer |
| [**listProviders**](PaymentApi.md#listproviders) | **GET** /v1/payment/providers | List payment providers by entity ID |



## getOrCreateStripeCustomer

> StripeCustomerResponse getOrCreateStripeCustomer(stripeCustomerRequest)

Get or create a Stripe customer

### Example

```ts
import {
  Configuration,
  PaymentApi,
} from '@tachyon/sdk';
import type { GetOrCreateStripeCustomerRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new PaymentApi();

  const body = {
    // StripeCustomerRequest
    stripeCustomerRequest: ...,
  } satisfies GetOrCreateStripeCustomerRequest;

  try {
    const data = await api.getOrCreateStripeCustomer(body);
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
| **stripeCustomerRequest** | [StripeCustomerRequest](StripeCustomerRequest.md) |  | |

### Return type

[**StripeCustomerResponse**](StripeCustomerResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Stripe customer |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listProviders

> ProviderListResponse listProviders(entityId)

List payment providers by entity ID

### Example

```ts
import {
  Configuration,
  PaymentApi,
} from '@tachyon/sdk';
import type { ListProvidersRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new PaymentApi();

  const body = {
    // string | Entity ID
    entityId: entityId_example,
  } satisfies ListProvidersRequest;

  try {
    const data = await api.listProviders(body);
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
| **entityId** | `string` | Entity ID | [Defaults to `undefined`] |

### Return type

[**ProviderListResponse**](ProviderListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Provider list |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

