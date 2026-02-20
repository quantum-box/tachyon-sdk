# tachyon_sdk.AuthUsersApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_user**](AuthUsersApi.md#create_user) | **POST** /auth/v1beta/users | 
[**get_user**](AuthUsersApi.md#get_user) | **GET** /v1/auth/users/{id} | Get a user by ID
[**list_users**](AuthUsersApi.md#list_users) | **GET** /v1/auth/users | List all users in an operator


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

