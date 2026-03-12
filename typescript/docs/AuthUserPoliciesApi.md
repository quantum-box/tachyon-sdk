# AuthUserPoliciesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**attachUserPolicy**](AuthUserPoliciesApi.md#attachuserpolicyoperation) | **POST** /v1/auth/user-policies/attach | Attach a policy to a user |
| [**attachUserPolicyWithScope**](AuthUserPoliciesApi.md#attachuserpolicywithscopeoperation) | **POST** /v1/auth/user-policies/attach-with-scope | Attach a policy to a user with resource scope |
| [**detachUserPolicy**](AuthUserPoliciesApi.md#detachuserpolicyoperation) | **POST** /v1/auth/user-policies/detach | Detach a policy from a user |
| [**detachUserPolicyWithScope**](AuthUserPoliciesApi.md#detachuserpolicywithscopeoperation) | **POST** /v1/auth/user-policies/detach-with-scope | Detach a scoped policy from a user |
| [**findUserPolicyMappings**](AuthUserPoliciesApi.md#finduserpolicymappings) | **GET** /v1/auth/user-policy-mappings | Find user policy mappings by resource scope |
| [**listUserPolicies**](AuthUserPoliciesApi.md#listuserpolicies) | **GET** /v1/auth/users/{user_id}/policies | List policies attached to a user |



## attachUserPolicy

> attachUserPolicy(attachUserPolicyRequest)

Attach a policy to a user

### Example

```ts
import {
  Configuration,
  AuthUserPoliciesApi,
} from '@tachyon/sdk';
import type { AttachUserPolicyOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUserPoliciesApi();

  const body = {
    // AttachUserPolicyRequest
    attachUserPolicyRequest: ...,
  } satisfies AttachUserPolicyOperationRequest;

  try {
    const data = await api.attachUserPolicy(body);
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
| **attachUserPolicyRequest** | [AttachUserPolicyRequest](AttachUserPolicyRequest.md) |  | |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: Not defined


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Policy attached |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## attachUserPolicyWithScope

> attachUserPolicyWithScope(attachUserPolicyWithScopeRequest)

Attach a policy to a user with resource scope

### Example

```ts
import {
  Configuration,
  AuthUserPoliciesApi,
} from '@tachyon/sdk';
import type { AttachUserPolicyWithScopeOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUserPoliciesApi();

  const body = {
    // AttachUserPolicyWithScopeRequest
    attachUserPolicyWithScopeRequest: ...,
  } satisfies AttachUserPolicyWithScopeOperationRequest;

  try {
    const data = await api.attachUserPolicyWithScope(body);
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
| **attachUserPolicyWithScopeRequest** | [AttachUserPolicyWithScopeRequest](AttachUserPolicyWithScopeRequest.md) |  | |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: Not defined


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Scoped policy attached |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## detachUserPolicy

> detachUserPolicy(detachUserPolicyRequest)

Detach a policy from a user

### Example

```ts
import {
  Configuration,
  AuthUserPoliciesApi,
} from '@tachyon/sdk';
import type { DetachUserPolicyOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUserPoliciesApi();

  const body = {
    // DetachUserPolicyRequest
    detachUserPolicyRequest: ...,
  } satisfies DetachUserPolicyOperationRequest;

  try {
    const data = await api.detachUserPolicy(body);
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
| **detachUserPolicyRequest** | [DetachUserPolicyRequest](DetachUserPolicyRequest.md) |  | |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: Not defined


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Policy detached |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## detachUserPolicyWithScope

> detachUserPolicyWithScope(detachUserPolicyWithScopeRequest)

Detach a scoped policy from a user

### Example

```ts
import {
  Configuration,
  AuthUserPoliciesApi,
} from '@tachyon/sdk';
import type { DetachUserPolicyWithScopeOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUserPoliciesApi();

  const body = {
    // DetachUserPolicyWithScopeRequest
    detachUserPolicyWithScopeRequest: ...,
  } satisfies DetachUserPolicyWithScopeOperationRequest;

  try {
    const data = await api.detachUserPolicyWithScope(body);
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
| **detachUserPolicyWithScopeRequest** | [DetachUserPolicyWithScopeRequest](DetachUserPolicyWithScopeRequest.md) |  | |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: Not defined


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Scoped policy detached |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## findUserPolicyMappings

> UserPolicyMappingListResponse findUserPolicyMappings(tenantId, resourceScope)

Find user policy mappings by resource scope

### Example

```ts
import {
  Configuration,
  AuthUserPoliciesApi,
} from '@tachyon/sdk';
import type { FindUserPolicyMappingsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUserPoliciesApi();

  const body = {
    // string | Tenant ID
    tenantId: tenantId_example,
    // string | Resource scope in TRN format
    resourceScope: resourceScope_example,
  } satisfies FindUserPolicyMappingsRequest;

  try {
    const data = await api.findUserPolicyMappings(body);
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
| **tenantId** | `string` | Tenant ID | [Defaults to `undefined`] |
| **resourceScope** | `string` | Resource scope in TRN format | [Defaults to `undefined`] |

### Return type

[**UserPolicyMappingListResponse**](UserPolicyMappingListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User policy mappings |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listUserPolicies

> UserPolicyListResponse listUserPolicies(userId, tenantId)

List policies attached to a user

### Example

```ts
import {
  Configuration,
  AuthUserPoliciesApi,
} from '@tachyon/sdk';
import type { ListUserPoliciesRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUserPoliciesApi();

  const body = {
    // string | User ID
    userId: userId_example,
    // string | Optional tenant ID filter (optional)
    tenantId: tenantId_example,
  } satisfies ListUserPoliciesRequest;

  try {
    const data = await api.listUserPolicies(body);
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
| **userId** | `string` | User ID | [Defaults to `undefined`] |
| **tenantId** | `string` | Optional tenant ID filter | [Optional] [Defaults to `undefined`] |

### Return type

[**UserPolicyListResponse**](UserPolicyListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User policy list |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

