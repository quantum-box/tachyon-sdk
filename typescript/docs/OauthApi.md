# OauthApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**callback**](OauthApi.md#callback) | **GET** /oauth/{provider_name}/callback | OAuth callback handler for specified provider |
| [**connect**](OauthApi.md#connect) | **GET** /oauth/{provider_name}/connect | Get OAuth authorization URL for specified provider |



## callback

> OAuthCallbackResponse callback(providerName, code, state)

OAuth callback handler for specified provider

### Example

```ts
import {
  Configuration,
  OauthApi,
} from '@tachyon/sdk';
import type { CallbackRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OauthApi();

  const body = {
    // string | OAuth provider name
    providerName: providerName_example,
    // string | Authorization code from provider
    code: code_example,
    // string | State parameter for CSRF protection
    state: state_example,
  } satisfies CallbackRequest;

  try {
    const data = await api.callback(body);
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
| **providerName** | `string` | OAuth provider name | [Defaults to `undefined`] |
| **code** | `string` | Authorization code from provider | [Defaults to `undefined`] |
| **state** | `string` | State parameter for CSRF protection | [Defaults to `undefined`] |

### Return type

[**OAuthCallbackResponse**](OAuthCallbackResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Successfully connected to provider |  -  |
| **400** | Failed to exchange code for token |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## connect

> AuthUrlResponse connect(providerName)

Get OAuth authorization URL for specified provider

### Example

```ts
import {
  Configuration,
  OauthApi,
} from '@tachyon/sdk';
import type { ConnectRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OauthApi();

  const body = {
    // string | OAuth provider name
    providerName: providerName_example,
  } satisfies ConnectRequest;

  try {
    const data = await api.connect(body);
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
| **providerName** | `string` | OAuth provider name | [Defaults to `undefined`] |

### Return type

[**AuthUrlResponse**](AuthUrlResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Successfully generated authorization URL |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

