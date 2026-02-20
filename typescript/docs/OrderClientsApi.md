# OrderClientsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createClient**](OrderClientsApi.md#createclientoperation) | **POST** /v1/order/clients | Create a new client |
| [**getClient**](OrderClientsApi.md#getclient) | **GET** /v1/order/clients/{id} | Get a client by ID |
| [**listClients**](OrderClientsApi.md#listclients) | **GET** /v1/order/clients | List all clients |



## createClient

> ClientResponse createClient(createClientRequest)

Create a new client

### Example

```ts
import {
  Configuration,
  OrderClientsApi,
} from '@tachyon/sdk';
import type { CreateClientOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderClientsApi();

  const body = {
    // CreateClientRequest
    createClientRequest: ...,
  } satisfies CreateClientOperationRequest;

  try {
    const data = await api.createClient(body);
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
| **createClientRequest** | [CreateClientRequest](CreateClientRequest.md) |  | |

### Return type

[**ClientResponse**](ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Client created |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getClient

> ClientResponse getClient(id)

Get a client by ID

### Example

```ts
import {
  Configuration,
  OrderClientsApi,
} from '@tachyon/sdk';
import type { GetClientRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderClientsApi();

  const body = {
    // string | Client ID
    id: id_example,
  } satisfies GetClientRequest;

  try {
    const data = await api.getClient(body);
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
| **id** | `string` | Client ID | [Defaults to `undefined`] |

### Return type

[**ClientResponse**](ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Client found |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listClients

> ClientListResponse listClients()

List all clients

### Example

```ts
import {
  Configuration,
  OrderClientsApi,
} from '@tachyon/sdk';
import type { ListClientsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderClientsApi();

  try {
    const data = await api.listClients();
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

[**ClientListResponse**](ClientListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Client list |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

