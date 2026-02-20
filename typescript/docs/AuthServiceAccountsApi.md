# AuthServiceAccountsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createServiceAccount**](AuthServiceAccountsApi.md#createserviceaccountoperation) | **POST** /v1/auth/service-accounts | Create a new service account |
| [**deleteServiceAccount**](AuthServiceAccountsApi.md#deleteserviceaccount) | **DELETE** /v1/auth/service-accounts/{id} | Delete a service account |
| [**getServiceAccount**](AuthServiceAccountsApi.md#getserviceaccount) | **GET** /v1/auth/service-accounts/{id} | Get a service account by ID |
| [**listServiceAccounts**](AuthServiceAccountsApi.md#listserviceaccounts) | **GET** /v1/auth/service-accounts | List all service accounts |
| [**updateServiceAccount**](AuthServiceAccountsApi.md#updateserviceaccountoperation) | **PUT** /v1/auth/service-accounts/{id} | Update a service account |



## createServiceAccount

> ServiceAccountResponse createServiceAccount(createServiceAccountRequest)

Create a new service account

### Example

```ts
import {
  Configuration,
  AuthServiceAccountsApi,
} from '@tachyon/sdk';
import type { CreateServiceAccountOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthServiceAccountsApi();

  const body = {
    // CreateServiceAccountRequest
    createServiceAccountRequest: ...,
  } satisfies CreateServiceAccountOperationRequest;

  try {
    const data = await api.createServiceAccount(body);
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
| **createServiceAccountRequest** | [CreateServiceAccountRequest](CreateServiceAccountRequest.md) |  | |

### Return type

[**ServiceAccountResponse**](ServiceAccountResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Service account created |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deleteServiceAccount

> DeleteServiceAccountResponse deleteServiceAccount(id)

Delete a service account

### Example

```ts
import {
  Configuration,
  AuthServiceAccountsApi,
} from '@tachyon/sdk';
import type { DeleteServiceAccountRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthServiceAccountsApi();

  const body = {
    // string | Service account ID
    id: id_example,
  } satisfies DeleteServiceAccountRequest;

  try {
    const data = await api.deleteServiceAccount(body);
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
| **id** | `string` | Service account ID | [Defaults to `undefined`] |

### Return type

[**DeleteServiceAccountResponse**](DeleteServiceAccountResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Service account deleted |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getServiceAccount

> ServiceAccountResponse getServiceAccount(id, operatorId)

Get a service account by ID

### Example

```ts
import {
  Configuration,
  AuthServiceAccountsApi,
} from '@tachyon/sdk';
import type { GetServiceAccountRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthServiceAccountsApi();

  const body = {
    // string | Service account ID
    id: id_example,
    // string | Operator ID
    operatorId: operatorId_example,
  } satisfies GetServiceAccountRequest;

  try {
    const data = await api.getServiceAccount(body);
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
| **id** | `string` | Service account ID | [Defaults to `undefined`] |
| **operatorId** | `string` | Operator ID | [Defaults to `undefined`] |

### Return type

[**ServiceAccountResponse**](ServiceAccountResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Service account found |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listServiceAccounts

> ServiceAccountListResponse listServiceAccounts(operatorId)

List all service accounts

### Example

```ts
import {
  Configuration,
  AuthServiceAccountsApi,
} from '@tachyon/sdk';
import type { ListServiceAccountsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthServiceAccountsApi();

  const body = {
    // string | Operator ID
    operatorId: operatorId_example,
  } satisfies ListServiceAccountsRequest;

  try {
    const data = await api.listServiceAccounts(body);
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
| **operatorId** | `string` | Operator ID | [Defaults to `undefined`] |

### Return type

[**ServiceAccountListResponse**](ServiceAccountListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Service account list |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateServiceAccount

> ServiceAccountResponse updateServiceAccount(id, updateServiceAccountRequest)

Update a service account

### Example

```ts
import {
  Configuration,
  AuthServiceAccountsApi,
} from '@tachyon/sdk';
import type { UpdateServiceAccountOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthServiceAccountsApi();

  const body = {
    // string | Service account ID
    id: id_example,
    // UpdateServiceAccountRequest
    updateServiceAccountRequest: ...,
  } satisfies UpdateServiceAccountOperationRequest;

  try {
    const data = await api.updateServiceAccount(body);
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
| **id** | `string` | Service account ID | [Defaults to `undefined`] |
| **updateServiceAccountRequest** | [UpdateServiceAccountRequest](UpdateServiceAccountRequest.md) |  | |

### Return type

[**ServiceAccountResponse**](ServiceAccountResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Service account updated |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

