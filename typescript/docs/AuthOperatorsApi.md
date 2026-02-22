# AuthOperatorsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createOperator**](AuthOperatorsApi.md#createoperatoroperation) | **POST** /v1/auth/operators | Create an operator under a platform |
| [**findOperatorsByUser**](AuthOperatorsApi.md#findoperatorsbyuser) | **GET** /v1/auth/operators/by-user | Find operators accessible to a user under a platform |
| [**getOperatorByAlias**](AuthOperatorsApi.md#getoperatorbyalias) | **GET** /v1/auth/operators/by-alias | Get an operator by alias within a platform |
| [**getOperatorById**](AuthOperatorsApi.md#getoperatorbyid) | **GET** /v1/auth/operators/{id} | Get an operator by ID |



## createOperator

> CreateOperatorResponse createOperator(createOperatorRequest)

Create an operator under a platform

### Example

```ts
import {
  Configuration,
  AuthOperatorsApi,
} from '@tachyon/sdk';
import type { CreateOperatorOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOperatorsApi();

  const body = {
    // CreateOperatorRequest
    createOperatorRequest: ...,
  } satisfies CreateOperatorOperationRequest;

  try {
    const data = await api.createOperator(body);
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
| **createOperatorRequest** | [CreateOperatorRequest](CreateOperatorRequest.md) |  | |

### Return type

[**CreateOperatorResponse**](CreateOperatorResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Operator created |  -  |
| **400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## findOperatorsByUser

> OperatorListResponse findOperatorsByUser(platformId, userId)

Find operators accessible to a user under a platform

### Example

```ts
import {
  Configuration,
  AuthOperatorsApi,
} from '@tachyon/sdk';
import type { FindOperatorsByUserRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOperatorsApi();

  const body = {
    // string | Platform ID
    platformId: platformId_example,
    // string | User ID
    userId: userId_example,
  } satisfies FindOperatorsByUserRequest;

  try {
    const data = await api.findOperatorsByUser(body);
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
| **platformId** | `string` | Platform ID | [Defaults to `undefined`] |
| **userId** | `string` | User ID | [Defaults to `undefined`] |

### Return type

[**OperatorListResponse**](OperatorListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Operators found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getOperatorByAlias

> OperatorResponse getOperatorByAlias(platformId, alias)

Get an operator by alias within a platform

### Example

```ts
import {
  Configuration,
  AuthOperatorsApi,
} from '@tachyon/sdk';
import type { GetOperatorByAliasRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOperatorsApi();

  const body = {
    // string | Platform ID
    platformId: platformId_example,
    // string | Operator alias (username)
    alias: alias_example,
  } satisfies GetOperatorByAliasRequest;

  try {
    const data = await api.getOperatorByAlias(body);
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
| **platformId** | `string` | Platform ID | [Defaults to `undefined`] |
| **alias** | `string` | Operator alias (username) | [Defaults to `undefined`] |

### Return type

[**OperatorResponse**](OperatorResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Operator found |  -  |
| **404** | Operator not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getOperatorById

> OperatorResponse getOperatorById(id)

Get an operator by ID

### Example

```ts
import {
  Configuration,
  AuthOperatorsApi,
} from '@tachyon/sdk';
import type { GetOperatorByIdRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthOperatorsApi();

  const body = {
    // string | Operator ID
    id: id_example,
  } satisfies GetOperatorByIdRequest;

  try {
    const data = await api.getOperatorById(body);
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
| **id** | `string` | Operator ID | [Defaults to `undefined`] |

### Return type

[**OperatorResponse**](OperatorResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Operator found |  -  |
| **404** | Operator not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

