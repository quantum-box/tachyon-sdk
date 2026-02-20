# AuthAPIKeysApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createApiKey**](AuthAPIKeysApi.md#createapikeyoperation) | **POST** /v1/auth/service-accounts/{service_account_id}/api-keys | Create a new API key for a service account |
| [**listApiKeys**](AuthAPIKeysApi.md#listapikeys) | **GET** /v1/auth/service-accounts/{service_account_id}/api-keys | List API keys for a service account |



## createApiKey

> ApiKeyResponse createApiKey(serviceAccountId, createApiKeyRequest)

Create a new API key for a service account

### Example

```ts
import {
  Configuration,
  AuthAPIKeysApi,
} from '@tachyon/sdk';
import type { CreateApiKeyOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthAPIKeysApi();

  const body = {
    // string | Service account ID
    serviceAccountId: serviceAccountId_example,
    // CreateApiKeyRequest
    createApiKeyRequest: ...,
  } satisfies CreateApiKeyOperationRequest;

  try {
    const data = await api.createApiKey(body);
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
| **serviceAccountId** | `string` | Service account ID | [Defaults to `undefined`] |
| **createApiKeyRequest** | [CreateApiKeyRequest](CreateApiKeyRequest.md) |  | |

### Return type

[**ApiKeyResponse**](ApiKeyResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | API key created |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listApiKeys

> ApiKeyListResponse listApiKeys(serviceAccountId, operatorId)

List API keys for a service account

### Example

```ts
import {
  Configuration,
  AuthAPIKeysApi,
} from '@tachyon/sdk';
import type { ListApiKeysRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthAPIKeysApi();

  const body = {
    // string | Service account ID
    serviceAccountId: serviceAccountId_example,
    // string | Operator ID
    operatorId: operatorId_example,
  } satisfies ListApiKeysRequest;

  try {
    const data = await api.listApiKeys(body);
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
| **serviceAccountId** | `string` | Service account ID | [Defaults to `undefined`] |
| **operatorId** | `string` | Operator ID | [Defaults to `undefined`] |

### Return type

[**ApiKeyListResponse**](ApiKeyListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | API key list |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

