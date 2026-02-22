# AuthUsersApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**addUserToTenant**](AuthUsersApi.md#addusertotenantoperation) | **POST** /v1/auth/users/{user_id}/tenants | Add a user to a tenant (grant tenant access) |
| [**createUser**](AuthUsersApi.md#createuseroperation) | **POST** /auth/v1beta/users |  |
| [**findUserByUsername**](AuthUsersApi.md#finduserbyusername) | **GET** /v1/auth/users/search/by-username | Find a user by username |
| [**getUser**](AuthUsersApi.md#getuser) | **GET** /v1/auth/users/{id} | Get a user by ID |
| [**inviteUser**](AuthUsersApi.md#inviteuseroperation) | **POST** /v1/auth/users/invite | Invite a user to a tenant |
| [**listUsers**](AuthUsersApi.md#listusers) | **GET** /v1/auth/users | List all users in an operator |
| [**updateUserRole**](AuthUsersApi.md#updateuserroleoperation) | **PUT** /v1/auth/users/{user_id}/role | Update a user\&#39;s role in a specific tenant |



## addUserToTenant

> UserResponse addUserToTenant(userId, addUserToTenantRequest)

Add a user to a tenant (grant tenant access)

### Example

```ts
import {
  Configuration,
  AuthUsersApi,
} from '@tachyon/sdk';
import type { AddUserToTenantOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUsersApi();

  const body = {
    // string | User ID
    userId: userId_example,
    // AddUserToTenantRequest
    addUserToTenantRequest: ...,
  } satisfies AddUserToTenantOperationRequest;

  try {
    const data = await api.addUserToTenant(body);
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
| **addUserToTenantRequest** | [AddUserToTenantRequest](AddUserToTenantRequest.md) |  | |

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User added to tenant |  -  |
| **404** | User not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createUser

> CreateUserResponse createUser(createUserRequest)



### Example

```ts
import {
  Configuration,
  AuthUsersApi,
} from '@tachyon/sdk';
import type { CreateUserOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUsersApi();

  const body = {
    // CreateUserRequest
    createUserRequest: ...,
  } satisfies CreateUserOperationRequest;

  try {
    const data = await api.createUser(body);
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
| **createUserRequest** | [CreateUserRequest](CreateUserRequest.md) |  | |

### Return type

[**CreateUserResponse**](CreateUserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User created successfully |  -  |
| **400** | Bad request |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## findUserByUsername

> UserResponse findUserByUsername(username)

Find a user by username

### Example

```ts
import {
  Configuration,
  AuthUsersApi,
} from '@tachyon/sdk';
import type { FindUserByUsernameRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUsersApi();

  const body = {
    // string | Username to search for
    username: username_example,
  } satisfies FindUserByUsernameRequest;

  try {
    const data = await api.findUserByUsername(body);
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
| **username** | `string` | Username to search for | [Defaults to `undefined`] |

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User found |  -  |
| **404** | User not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getUser

> UserResponse getUser(id)

Get a user by ID

### Example

```ts
import {
  Configuration,
  AuthUsersApi,
} from '@tachyon/sdk';
import type { GetUserRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUsersApi();

  const body = {
    // string | User ID
    id: id_example,
  } satisfies GetUserRequest;

  try {
    const data = await api.getUser(body);
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
| **id** | `string` | User ID | [Defaults to `undefined`] |

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User found |  -  |
| **404** | User not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## inviteUser

> UserResponse inviteUser(inviteUserRequest)

Invite a user to a tenant

### Example

```ts
import {
  Configuration,
  AuthUsersApi,
} from '@tachyon/sdk';
import type { InviteUserOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUsersApi();

  const body = {
    // InviteUserRequest
    inviteUserRequest: ...,
  } satisfies InviteUserOperationRequest;

  try {
    const data = await api.inviteUser(body);
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
| **inviteUserRequest** | [InviteUserRequest](InviteUserRequest.md) |  | |

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User invited |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listUsers

> UserListResponse listUsers(operatorId)

List all users in an operator

### Example

```ts
import {
  Configuration,
  AuthUsersApi,
} from '@tachyon/sdk';
import type { ListUsersRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUsersApi();

  const body = {
    // string | Operator ID to list users for
    operatorId: operatorId_example,
  } satisfies ListUsersRequest;

  try {
    const data = await api.listUsers(body);
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
| **operatorId** | `string` | Operator ID to list users for | [Defaults to `undefined`] |

### Return type

[**UserListResponse**](UserListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | User list |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateUserRole

> UserResponse updateUserRole(userId, updateUserRoleRequest)

Update a user\&#39;s role in a specific tenant

### Example

```ts
import {
  Configuration,
  AuthUsersApi,
} from '@tachyon/sdk';
import type { UpdateUserRoleOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AuthUsersApi();

  const body = {
    // string | User ID
    userId: userId_example,
    // UpdateUserRoleRequest
    updateUserRoleRequest: ...,
  } satisfies UpdateUserRoleOperationRequest;

  try {
    const data = await api.updateUserRole(body);
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
| **updateUserRoleRequest** | [UpdateUserRoleRequest](UpdateUserRoleRequest.md) |  | |

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Role updated |  -  |
| **400** | Bad request |  -  |
| **404** | User not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

