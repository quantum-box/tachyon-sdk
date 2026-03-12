# ModelsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getModels**](ModelsApi.md#getmodels) | **GET** /v1/llms/models | List available LLM models |



## getModels

> ModelsResponse getModels(xOperatorId, authorization, supportedFeature, requireAgentProduct)

List available LLM models

Returns all LLM models available for the operator. Models can be filtered by supported features such as &#x60;streaming&#x60;, &#x60;function_calling&#x60;, &#x60;vision&#x60;, or &#x60;agent&#x60;. By default, only agent-capable models are returned (&#x60;require_agent_product&#x3D;true&#x60;).

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
    xOperatorId: tn_01hjryxysgey07h5jz5wagqj0m,
    // string | Bearer token for authentication
    authorization: Bearer dummy-token,
    // Array<SupportedFeature> | Filter models by supported feature. Accepts a single value or comma-separated list (e.g. `?supported_feature=streaming,agent`). When multiple features are specified, only models supporting **all** of them are returned. (optional)
    supportedFeature: ["streaming","agent"],
    // boolean | When `true` (default), only models that support the `agent` feature are returned. The `agent` filter is automatically appended if not already present in `supported_feature`. (optional)
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
| **authorization** | `string` | Bearer token for authentication | [Defaults to `undefined`] |
| **supportedFeature** | `Array<SupportedFeature>` | Filter models by supported feature. Accepts a single value or comma-separated list (e.g. &#x60;?supported_feature&#x3D;streaming,agent&#x60;). When multiple features are specified, only models supporting **all** of them are returned. | [Optional] |
| **requireAgentProduct** | `boolean` | When &#x60;true&#x60; (default), only models that support the &#x60;agent&#x60; feature are returned. The &#x60;agent&#x60; filter is automatically appended if not already present in &#x60;supported_feature&#x60;. | [Optional] [Defaults to `undefined`] |

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
| **200** | Successfully retrieved model list |  -  |
| **401** | Unauthorized — missing or invalid Authorization header |  -  |
| **403** | Forbidden — operator does not have permission |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

