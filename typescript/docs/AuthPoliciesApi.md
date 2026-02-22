# AuthPoliciesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**checkPolicyForResource**](AuthPoliciesApi.md#checkpolicyforresourceoperation) | **POST** /v1/auth/policies/check-for-resource | Check permission for a specific resource |
| [**evaluatePoliciesBatch**](AuthPoliciesApi.md#evaluatepoliciesbatchoperation) | **POST** /v1/auth/policies/check | Evaluate multiple policy actions in batch |
| [**getPolicy**](AuthPoliciesApi.md#getpolicy) | **GET** /v1/auth/policies/{id} | Get a policy by ID |
| [**listActions**](AuthPoliciesApi.md#listactions) | **GET** /v1/auth/actions | List all registered actions |



## checkPolicyForResource

> CheckPolicyForResourceResponse checkPolicyForResource(checkPolicyForResourceRequest)

Check permission for a specific resource

### Example

```ts
import {
  Configuration,
  AuthPoliciesApi,
} from '@tachyon/sdk';
import type { CheckPolicyForResourceOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthPoliciesApi();

  const body = {
    // CheckPolicyForResourceRequest
    checkPolicyForResourceRequest: ...,
  } satisfies CheckPolicyForResourceOperationRequest;

  try {
    const data = await api.checkPolicyForResource(body);
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
| **checkPolicyForResourceRequest** | [CheckPolicyForResourceRequest](CheckPolicyForResourceRequest.md) |  | |

### Return type

[**CheckPolicyForResourceResponse**](CheckPolicyForResourceResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Permission check result |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## evaluatePoliciesBatch

> EvaluatePoliciesBatchResponse evaluatePoliciesBatch(evaluatePoliciesBatchRequest)

Evaluate multiple policy actions in batch

### Example

```ts
import {
  Configuration,
  AuthPoliciesApi,
} from '@tachyon/sdk';
import type { EvaluatePoliciesBatchOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthPoliciesApi();

  const body = {
    // EvaluatePoliciesBatchRequest
    evaluatePoliciesBatchRequest: ...,
  } satisfies EvaluatePoliciesBatchOperationRequest;

  try {
    const data = await api.evaluatePoliciesBatch(body);
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
| **evaluatePoliciesBatchRequest** | [EvaluatePoliciesBatchRequest](EvaluatePoliciesBatchRequest.md) |  | |

### Return type

[**EvaluatePoliciesBatchResponse**](EvaluatePoliciesBatchResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Evaluation results |  -  |
| **400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getPolicy

> PolicyResponse getPolicy(id)

Get a policy by ID

### Example

```ts
import {
  Configuration,
  AuthPoliciesApi,
} from '@tachyon/sdk';
import type { GetPolicyRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthPoliciesApi();

  const body = {
    // string | Policy ID
    id: id_example,
  } satisfies GetPolicyRequest;

  try {
    const data = await api.getPolicy(body);
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
| **id** | `string` | Policy ID | [Defaults to `undefined`] |

### Return type

[**PolicyResponse**](PolicyResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Policy found |  -  |
| **404** | Policy not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listActions

> ActionListResponse listActions(context)

List all registered actions

### Example

```ts
import {
  Configuration,
  AuthPoliciesApi,
} from '@tachyon/sdk';
import type { ListActionsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthPoliciesApi();

  const body = {
    // string | Filter by context (optional)
    context: context_example,
  } satisfies ListActionsRequest;

  try {
    const data = await api.listActions(body);
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
| **context** | `string` | Filter by context | [Optional] [Defaults to `undefined`] |

### Return type

[**ActionListResponse**](ActionListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Action list |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

