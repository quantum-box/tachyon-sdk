# FeatureFlagsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**evaluateActions**](FeatureFlagsApi.md#evaluateactionsoperation) | **POST** /v1/feature-flags/actions/evaluate | TODO: add English documentation |



## evaluateActions

> EvaluateActionsResponse evaluateActions(evaluateActionsRequest)

TODO: add English documentation

### Example

```ts
import {
  Configuration,
  FeatureFlagsApi,
} from '@tachyon/sdk';
import type { EvaluateActionsOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new FeatureFlagsApi();

  const body = {
    // EvaluateActionsRequest
    evaluateActionsRequest: ...,
  } satisfies EvaluateActionsOperationRequest;

  try {
    const data = await api.evaluateActions(body);
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
| **evaluateActionsRequest** | [EvaluateActionsRequest](EvaluateActionsRequest.md) |  | |

### Return type

[**EvaluateActionsResponse**](EvaluateActionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | 評価結果 |  -  |
| **400** | 不正な入力 |  -  |
| **401** | 認証エラー |  -  |
| **403** | 権限不足 |  -  |
| **500** | 内部エラー |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

