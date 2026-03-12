# tachyon_sdk.AuthUsersApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_user_to_tenant**](AuthUsersApi.md#add_user_to_tenant) | **POST** /v1/auth/users/{user_id}/tenants | Add a user to a tenant (grant tenant access)
[**create_user**](AuthUsersApi.md#create_user) | **POST** /auth/v1beta/users | 
[**find_user_by_username**](AuthUsersApi.md#find_user_by_username) | **GET** /v1/auth/users/search/by-username | Find a user by username
[**get_user**](AuthUsersApi.md#get_user) | **GET** /v1/auth/users/{id} | Get a user by ID
[**invite_user**](AuthUsersApi.md#invite_user) | **POST** /v1/auth/users/invite | Invite a user to a tenant
[**list_users**](AuthUsersApi.md#list_users) | **GET** /v1/auth/users | List all users in an operator
[**update_user_role**](AuthUsersApi.md#update_user_role) | **PUT** /v1/auth/users/{user_id}/role | Update a user&#39;s role in a specific tenant


# **add_user_to_tenant**
> UserResponse add_user_to_tenant(user_id, add_user_to_tenant_request)

Add a user to a tenant (grant tenant access)

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.add_user_to_tenant_request import AddUserToTenantRequest
from tachyon_sdk.models.user_response import UserResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AuthUsersApi(api_client)
    user_id = 'user_id_example' # str | User ID
    add_user_to_tenant_request = tachyon_sdk.AddUserToTenantRequest() # AddUserToTenantRequest | 

    try:
        # Add a user to a tenant (grant tenant access)
        api_response = api_instance.add_user_to_tenant(user_id, add_user_to_tenant_request)
        print("The response of AuthUsersApi->add_user_to_tenant:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUsersApi->add_user_to_tenant: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **str**| User ID | 
 **add_user_to_tenant_request** | [**AddUserToTenantRequest**](AddUserToTenantRequest.md)|  | 

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User added to tenant |  -  |
**404** | User not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_user**
> CreateUserResponse create_user(create_user_request)

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_user_request import CreateUserRequest
from tachyon_sdk.models.create_user_response import CreateUserResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AuthUsersApi(api_client)
    create_user_request = tachyon_sdk.CreateUserRequest() # CreateUserRequest | 

    try:
        api_response = api_instance.create_user(create_user_request)
        print("The response of AuthUsersApi->create_user:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUsersApi->create_user: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_user_request** | [**CreateUserRequest**](CreateUserRequest.md)|  | 

### Return type

[**CreateUserResponse**](CreateUserResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User created successfully |  -  |
**400** | Bad request |  -  |
**500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **find_user_by_username**
> UserResponse find_user_by_username(username)

Find a user by username

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.user_response import UserResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AuthUsersApi(api_client)
    username = 'username_example' # str | Username to search for

    try:
        # Find a user by username
        api_response = api_instance.find_user_by_username(username)
        print("The response of AuthUsersApi->find_user_by_username:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUsersApi->find_user_by_username: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **username** | **str**| Username to search for | 

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User found |  -  |
**404** | User not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_user**
> UserResponse get_user(id)

Get a user by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.user_response import UserResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AuthUsersApi(api_client)
    id = 'id_example' # str | User ID

    try:
        # Get a user by ID
        api_response = api_instance.get_user(id)
        print("The response of AuthUsersApi->get_user:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUsersApi->get_user: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| User ID | 

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User found |  -  |
**404** | User not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **invite_user**
> UserResponse invite_user(invite_user_request)

Invite a user to a tenant

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.invite_user_request import InviteUserRequest
from tachyon_sdk.models.user_response import UserResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AuthUsersApi(api_client)
    invite_user_request = tachyon_sdk.InviteUserRequest() # InviteUserRequest | 

    try:
        # Invite a user to a tenant
        api_response = api_instance.invite_user(invite_user_request)
        print("The response of AuthUsersApi->invite_user:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUsersApi->invite_user: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **invite_user_request** | [**InviteUserRequest**](InviteUserRequest.md)|  | 

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User invited |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_users**
> UserListResponse list_users(operator_id)

List all users in an operator

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.user_list_response import UserListResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AuthUsersApi(api_client)
    operator_id = 'operator_id_example' # str | Operator ID to list users for

    try:
        # List all users in an operator
        api_response = api_instance.list_users(operator_id)
        print("The response of AuthUsersApi->list_users:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUsersApi->list_users: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **operator_id** | **str**| Operator ID to list users for | 

### Return type

[**UserListResponse**](UserListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User list |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_user_role**
> UserResponse update_user_role(user_id, update_user_role_request)

Update a user's role in a specific tenant

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.update_user_role_request import UpdateUserRoleRequest
from tachyon_sdk.models.user_response import UserResponse
from tachyon_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = tachyon_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
with tachyon_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = tachyon_sdk.AuthUsersApi(api_client)
    user_id = 'user_id_example' # str | User ID
    update_user_role_request = tachyon_sdk.UpdateUserRoleRequest() # UpdateUserRoleRequest | 

    try:
        # Update a user's role in a specific tenant
        api_response = api_instance.update_user_role(user_id, update_user_role_request)
        print("The response of AuthUsersApi->update_user_role:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUsersApi->update_user_role: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **str**| User ID | 
 **update_user_role_request** | [**UpdateUserRoleRequest**](UpdateUserRoleRequest.md)|  | 

### Return type

[**UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Role updated |  -  |
**400** | Bad request |  -  |
**404** | User not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

