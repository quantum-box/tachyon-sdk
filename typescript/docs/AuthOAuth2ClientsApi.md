# AuthOAuth2ClientsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createClient**](AuthOAuth2ClientsApi.md#createclientoperation) | **POST** /v1/auth/oauth2-clients | Create a new OAuth2 client |
| [**getClient**](AuthOAuth2ClientsApi.md#getclient) | **GET** /v1/auth/oauth2-clients/{id} | Get an OAuth2 client by ID |
| [**listClients**](AuthOAuth2ClientsApi.md#listclients) | **GET** /v1/auth/oauth2-clients | List all OAuth2 clients for the current tenant |
| [**revokeClient**](AuthOAuth2ClientsApi.md#revokeclient) | **POST** /v1/auth/oauth2-clients/{id}/revoke | Revoke an OAuth2 client |
| [**rotateSecret**](AuthOAuth2ClientsApi.md#rotatesecret) | **POST** /v1/auth/oauth2-clients/{id}/rotate-secret | Rotate an OAuth2 client secret |
| [**updateClient**](AuthOAuth2ClientsApi.md#updateclientoperation) | **PUT** /v1/auth/oauth2-clients/{id} | Update an OAuth2 client |



## createClient

> CreateClientResponse createClient(createClientRequest)

Create a new OAuth2 client

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { CreateClientOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

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

[**CreateClientResponse**](CreateClientResponse.md)

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

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getClient

> ClientResponse getClient(id)

Get an OAuth2 client by ID

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { GetClientRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // string | OAuth2 client ID
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
| **id** | `string` | OAuth2 client ID | [Defaults to `undefined`] |

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
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listClients

> ClientListResponse listClients()

List all OAuth2 clients for the current tenant

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { ListClientsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

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

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## revokeClient

> revokeClient(id)

Revoke an OAuth2 client

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { RevokeClientRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // string | OAuth2 client ID
    id: id_example,
  } satisfies RevokeClientRequest;

  try {
    const data = await api.revokeClient(body);
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
| **id** | `string` | OAuth2 client ID | [Defaults to `undefined`] |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **204** | Client revoked |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## rotateSecret

> RotateSecretResponse rotateSecret(id)

Rotate an OAuth2 client secret

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { RotateSecretRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // string | OAuth2 client ID
    id: id_example,
  } satisfies RotateSecretRequest;

  try {
    const data = await api.rotateSecret(body);
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
| **id** | `string` | OAuth2 client ID | [Defaults to `undefined`] |

### Return type

[**RotateSecretResponse**](RotateSecretResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Secret rotated |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateClient

> ClientResponse updateClient(id, updateClientRequest)

Update an OAuth2 client

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { UpdateClientOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // string | OAuth2 client ID
    id: id_example,
    // UpdateClientRequest
    updateClientRequest: ...,
  } satisfies UpdateClientOperationRequest;

  try {
    const data = await api.updateClient(body);
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
| **id** | `string` | OAuth2 client ID | [Defaults to `undefined`] |
| **updateClientRequest** | [UpdateClientRequest](UpdateClientRequest.md) |  | |

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
| **200** | Client updated |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

