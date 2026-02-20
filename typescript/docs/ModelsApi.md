# ModelsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getModels**](ModelsApi.md#getmodels) | **GET** /v1/llms/models | Get list of supported models |



## getModels

> ModelsResponse getModels(xOperatorId, authorization, supportedFeature, requireAgentProduct)

Get list of supported models

Get list of all LLM models available for agents.

### Example

```ts
import {
  Configuration,
  ModelsApi,
} from '@tachyon/sdk';
import type { GetModelsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new ModelsApi();

  const body = {
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // Array<string> | TODO: add English documentation (optional)
    supportedFeature: ...,
    // boolean | TODO: add English documentation (optional)
    requireAgentProduct: true,
  } satisfies GetModelsRequest;

  try {
    const data = await api.getModels(body);
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
| **xOperatorId** | `string` | Operator ID | [Defaults to `undefined`] |
| **authorization** | `string` | Authorization | [Defaults to `undefined`] |
| **supportedFeature** | `Array<string>` | TODO: add English documentation | [Optional] |
| **requireAgentProduct** | `boolean` | TODO: add English documentation | [Optional] [Defaults to `undefined`] |

### Return type

[**ModelsResponse**](ModelsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | モデル一覧の取得に成功 |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

