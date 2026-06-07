# AuthOAuth2ClientsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createOauth2Client**](AuthOAuth2ClientsApi.md#createoauth2client) | **POST** /v1/auth/oauth2-clients | Create a new OAuth2 client |
| [**getOauth2Client**](AuthOAuth2ClientsApi.md#getoauth2client) | **GET** /v1/auth/oauth2-clients/{id} | Get an OAuth2 client by ID |
| [**listOauth2Clients**](AuthOAuth2ClientsApi.md#listoauth2clients) | **GET** /v1/auth/oauth2-clients | List all OAuth2 clients for the current tenant |
| [**revokeOauth2Client**](AuthOAuth2ClientsApi.md#revokeoauth2client) | **POST** /v1/auth/oauth2-clients/{id}/revoke | Revoke an OAuth2 client |
| [**rotateOauth2ClientSecret**](AuthOAuth2ClientsApi.md#rotateoauth2clientsecret) | **POST** /v1/auth/oauth2-clients/{id}/rotate-secret | Rotate an OAuth2 client secret |
| [**updateOauth2Client**](AuthOAuth2ClientsApi.md#updateoauth2client) | **PUT** /v1/auth/oauth2-clients/{id} | Update an OAuth2 client |



## createOauth2Client

> OAuth2CreateClientResponse createOauth2Client(oAuth2CreateClientRequest)

Create a new OAuth2 client

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { CreateOauth2ClientRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // OAuth2CreateClientRequest
    oAuth2CreateClientRequest: ...,
  } satisfies CreateOauth2ClientRequest;

  try {
    const data = await api.createOauth2Client(body);
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
| **oAuth2CreateClientRequest** | [OAuth2CreateClientRequest](OAuth2CreateClientRequest.md) |  | |

### Return type

[**OAuth2CreateClientResponse**](OAuth2CreateClientResponse.md)

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


## getOauth2Client

> OAuth2ClientResponse getOauth2Client(id)

Get an OAuth2 client by ID

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { GetOauth2ClientRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // string | OAuth2 client ID
    id: id_example,
  } satisfies GetOauth2ClientRequest;

  try {
    const data = await api.getOauth2Client(body);
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

[**OAuth2ClientResponse**](OAuth2ClientResponse.md)

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


## listOauth2Clients

> OAuth2ClientListResponse listOauth2Clients()

List all OAuth2 clients for the current tenant

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { ListOauth2ClientsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  try {
    const data = await api.listOauth2Clients();
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

[**OAuth2ClientListResponse**](OAuth2ClientListResponse.md)

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


## revokeOauth2Client

> revokeOauth2Client(id)

Revoke an OAuth2 client

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { RevokeOauth2ClientRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // string | OAuth2 client ID
    id: id_example,
  } satisfies RevokeOauth2ClientRequest;

  try {
    const data = await api.revokeOauth2Client(body);
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


## rotateOauth2ClientSecret

> RotateSecretResponse rotateOauth2ClientSecret(id)

Rotate an OAuth2 client secret

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { RotateOauth2ClientSecretRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // string | OAuth2 client ID
    id: id_example,
  } satisfies RotateOauth2ClientSecretRequest;

  try {
    const data = await api.rotateOauth2ClientSecret(body);
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


## updateOauth2Client

> OAuth2ClientResponse updateOauth2Client(id, oAuth2UpdateClientRequest)

Update an OAuth2 client

### Example

```ts
import {
  Configuration,
  AuthOAuth2ClientsApi,
} from '@tachyon/sdk';
import type { UpdateOauth2ClientRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ClientsApi();

  const body = {
    // string | OAuth2 client ID
    id: id_example,
    // OAuth2UpdateClientRequest
    oAuth2UpdateClientRequest: ...,
  } satisfies UpdateOauth2ClientRequest;

  try {
    const data = await api.updateOauth2Client(body);
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
| **oAuth2UpdateClientRequest** | [OAuth2UpdateClientRequest](OAuth2UpdateClientRequest.md) |  | |

### Return type

[**OAuth2ClientResponse**](OAuth2ClientResponse.md)

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

