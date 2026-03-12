# AuthOAuth2ConsentsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**listConsents**](AuthOAuth2ConsentsApi.md#listconsents) | **GET** /v1/auth/consents | GET /v1/auth/consents |
| [**revokeConsent**](AuthOAuth2ConsentsApi.md#revokeconsent) | **POST** /v1/auth/consents/{id}/revoke | POST /v1/auth/consents/:id/revoke |



## listConsents

> ConsentListResponse listConsents()

GET /v1/auth/consents

List all consents for the authenticated user.

### Example

```ts
import {
  Configuration,
  AuthOAuth2ConsentsApi,
} from '@tachyon/sdk';
import type { ListConsentsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ConsentsApi();

  try {
    const data = await api.listConsents();
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

[**ConsentListResponse**](ConsentListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Consent list |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## revokeConsent

> revokeConsent(id)

POST /v1/auth/consents/:id/revoke

Revoke a user consent by ID.

### Example

```ts
import {
  Configuration,
  AuthOAuth2ConsentsApi,
} from '@tachyon/sdk';
import type { RevokeConsentRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOAuth2ConsentsApi();

  const body = {
    // string | Consent ID
    id: id_example,
  } satisfies RevokeConsentRequest;

  try {
    const data = await api.revokeConsent(body);
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
| **id** | `string` | Consent ID | [Defaults to `undefined`] |

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
| **204** | Consent revoked |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

