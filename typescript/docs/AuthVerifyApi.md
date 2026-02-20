# AuthVerifyApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**signInWithPlatform**](AuthVerifyApi.md#signinwithplatformoperation) | **POST** /auth/v1beta/sign-in-with-platform |  |
| [**verify**](AuthVerifyApi.md#verifyoperation) | **POST** /auth/v1beta/verify |  |



## signInWithPlatform

> SignInWithPlatformResponse signInWithPlatform(signInWithPlatformRequest)



### Example

```ts
import {
  Configuration,
  AuthVerifyApi,
} from '@tachyon/sdk';
import type { SignInWithPlatformOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthVerifyApi();

  const body = {
    // SignInWithPlatformRequest
    signInWithPlatformRequest: ...,
  } satisfies SignInWithPlatformOperationRequest;

  try {
    const data = await api.signInWithPlatform(body);
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
| **signInWithPlatformRequest** | [SignInWithPlatformRequest](SignInWithPlatformRequest.md) |  | |

### Return type

[**SignInWithPlatformResponse**](SignInWithPlatformResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User signed in or created |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## verify

> VerifyResponse verify(verifyRequest)



### Example

```ts
import {
  Configuration,
  AuthVerifyApi,
} from '@tachyon/sdk';
import type { VerifyOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthVerifyApi();

  const body = {
    // VerifyRequest
    verifyRequest: ...,
  } satisfies VerifyOperationRequest;

  try {
    const data = await api.verify(body);
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
| **verifyRequest** | [VerifyRequest](VerifyRequest.md) |  | |

### Return type

[**VerifyResponse**](VerifyResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Successful response |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

