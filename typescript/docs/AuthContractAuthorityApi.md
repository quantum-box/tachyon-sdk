# AuthContractAuthorityApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**resolve**](AuthContractAuthorityApi.md#resolve) | **POST** /v1/auth/contract-authority/resolve |  |



## resolve

> ContractAuthorityResolveResponse resolve(contractAuthorityResolveRequest)



### Example

```ts
import {
  Configuration,
  AuthContractAuthorityApi,
} from '@tachyon/sdk';
import type { ResolveRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthContractAuthorityApi();

  const body = {
    // ContractAuthorityResolveRequest
    contractAuthorityResolveRequest: ...,
  } satisfies ResolveRequest;

  try {
    const data = await api.resolve(body);
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
| **contractAuthorityResolveRequest** | [ContractAuthorityResolveRequest](ContractAuthorityResolveRequest.md) |  | |

### Return type

[**ContractAuthorityResolveResponse**](ContractAuthorityResolveResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Current verified authority for the exact binding slot |  -  |
| **400** | Invalid request scope or shape |  -  |
| **403** | Caller denied or current proof unavailable |  -  |
| **503** | Authority source unavailable |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

