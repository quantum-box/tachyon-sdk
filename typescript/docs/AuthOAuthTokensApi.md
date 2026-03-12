# AuthOAuthTokensApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**deleteOauthToken**](AuthOAuthTokensApi.md#deleteoauthtoken) | **DELETE** /v1/auth/oauth-tokens/{provider} | Delete an OAuth token |
| [**getOauthTokenByProvider**](AuthOAuthTokensApi.md#getoauthtokenbyprovider) | **GET** /v1/auth/oauth-tokens/{provider} | Get an OAuth token by provider |
| [**listOauthTokens**](AuthOAuthTokensApi.md#listoauthtokens) | **GET** /v1/auth/oauth-tokens | List all OAuth tokens |
| [**saveOauthToken**](AuthOAuthTokensApi.md#saveoauthtoken) | **POST** /v1/auth/oauth-tokens | Save an OAuth token |



## deleteOauthToken

> deleteOauthToken(provider)

Delete an OAuth token

### Example

```ts
import {
  Configuration,
  AuthOAuthTokensApi,
} from '@tachyon/sdk';
import type { DeleteOauthTokenRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuthTokensApi();

  const body = {
    // string | OAuth provider name
    provider: provider_example,
  } satisfies DeleteOauthTokenRequest;

  try {
    const data = await api.deleteOauthToken(body);
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
| **provider** | `string` | OAuth provider name | [Defaults to `undefined`] |

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
| **200** | OAuth token deleted |  -  |
| **403** | Forbidden |  -  |
| **404** | Token not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getOauthTokenByProvider

> OAuthTokenDetailResponse getOauthTokenByProvider(provider)

Get an OAuth token by provider

### Example

```ts
import {
  Configuration,
  AuthOAuthTokensApi,
} from '@tachyon/sdk';
import type { GetOauthTokenByProviderRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuthTokensApi();

  const body = {
    // string | OAuth provider name
    provider: provider_example,
  } satisfies GetOauthTokenByProviderRequest;

  try {
    const data = await api.getOauthTokenByProvider(body);
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
| **provider** | `string` | OAuth provider name | [Defaults to `undefined`] |

### Return type

[**OAuthTokenDetailResponse**](OAuthTokenDetailResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | OAuth token found |  -  |
| **404** | Token not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listOauthTokens

> OAuthTokenListResponse listOauthTokens()

List all OAuth tokens

### Example

```ts
import {
  Configuration,
  AuthOAuthTokensApi,
} from '@tachyon/sdk';
import type { ListOauthTokensRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuthTokensApi();

  try {
    const data = await api.listOauthTokens();
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

[**OAuthTokenListResponse**](OAuthTokenListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | OAuth token list |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## saveOauthToken

> OAuthTokenResponse saveOauthToken(saveOAuthTokenRequest)

Save an OAuth token

### Example

```ts
import {
  Configuration,
  AuthOAuthTokensApi,
} from '@tachyon/sdk';
import type { SaveOauthTokenRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuthTokensApi();

  const body = {
    // SaveOAuthTokenRequest
    saveOAuthTokenRequest: ...,
  } satisfies SaveOauthTokenRequest;

  try {
    const data = await api.saveOauthToken(body);
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
| **saveOAuthTokenRequest** | [SaveOAuthTokenRequest](SaveOAuthTokenRequest.md) |  | |

### Return type

[**OAuthTokenResponse**](OAuthTokenResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | OAuth token saved |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

