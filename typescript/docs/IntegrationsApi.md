# IntegrationsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**connectIntegration**](IntegrationsApi.md#connectintegration) | **POST** /v1/integrations/{id}/connect | Initiate an OAuth connection to an integration. |
| [**disconnectIntegration**](IntegrationsApi.md#disconnectintegration) | **DELETE** /v1/integrations/connections/{id} | Disconnect an integration connection. |
| [**getIntegration**](IntegrationsApi.md#getintegration) | **GET** /v1/integrations/{id} | Get an integration by ID. |
| [**listConnections**](IntegrationsApi.md#listconnections) | **GET** /v1/integrations/connections | List all connections for the current tenant. |
| [**listIntegrations**](IntegrationsApi.md#listintegrations) | **GET** /v1/integrations | List all integrations in the marketplace. |



## connectIntegration

> ConnectResponse connectIntegration(id)

Initiate an OAuth connection to an integration.

### Example

```ts
import {
  Configuration,
  IntegrationsApi,
} from '@tachyon/sdk';
import type { ConnectIntegrationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new IntegrationsApi();

  const body = {
    // string | Integration ID
    id: id_example,
  } satisfies ConnectIntegrationRequest;

  try {
    const data = await api.connectIntegration(body);
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
| **id** | `string` | Integration ID | [Defaults to `undefined`] |

### Return type

[**ConnectResponse**](ConnectResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | OAuth authorization URL |  -  |
| **403** | Forbidden |  -  |
| **404** | Integration not found |  -  |
| **409** | Already connected |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## disconnectIntegration

> disconnectIntegration(id)

Disconnect an integration connection.

### Example

```ts
import {
  Configuration,
  IntegrationsApi,
} from '@tachyon/sdk';
import type { DisconnectIntegrationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new IntegrationsApi();

  const body = {
    // string | Connection ID
    id: id_example,
  } satisfies DisconnectIntegrationRequest;

  try {
    const data = await api.disconnectIntegration(body);
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
| **id** | `string` | Connection ID | [Defaults to `undefined`] |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **204** | Disconnected |  -  |
| **403** | Forbidden |  -  |
| **404** | Connection not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getIntegration

> IntegrationDetailResponse getIntegration(id)

Get an integration by ID.

### Example

```ts
import {
  Configuration,
  IntegrationsApi,
} from '@tachyon/sdk';
import type { GetIntegrationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new IntegrationsApi();

  const body = {
    // string | Integration ID
    id: id_example,
  } satisfies GetIntegrationRequest;

  try {
    const data = await api.getIntegration(body);
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
| **id** | `string` | Integration ID | [Defaults to `undefined`] |

### Return type

[**IntegrationDetailResponse**](IntegrationDetailResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Integration details |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listConnections

> ListConnectionsResponse listConnections()

List all connections for the current tenant.

### Example

```ts
import {
  Configuration,
  IntegrationsApi,
} from '@tachyon/sdk';
import type { ListConnectionsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new IntegrationsApi();

  try {
    const data = await api.listConnections();
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

[**ListConnectionsResponse**](ListConnectionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Connection list |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listIntegrations

> ListIntegrationsResponse listIntegrations(category)

List all integrations in the marketplace.

### Example

```ts
import {
  Configuration,
  IntegrationsApi,
} from '@tachyon/sdk';
import type { ListIntegrationsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new IntegrationsApi();

  const body = {
    // string | Filter by category (optional)
    category: category_example,
  } satisfies ListIntegrationsRequest;

  try {
    const data = await api.listIntegrations(body);
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
| **category** | `string` | Filter by category | [Optional] [Defaults to `undefined`] |

### Return type

[**ListIntegrationsResponse**](ListIntegrationsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Integration list |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

