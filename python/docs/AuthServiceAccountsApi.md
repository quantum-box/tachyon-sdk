# tachyon_sdk.AuthServiceAccountsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_service_account**](AuthServiceAccountsApi.md#create_service_account) | **POST** /v1/auth/service-accounts | Create a new service account
[**delete_service_account**](AuthServiceAccountsApi.md#delete_service_account) | **DELETE** /v1/auth/service-accounts/{id} | Delete a service account
[**get_service_account**](AuthServiceAccountsApi.md#get_service_account) | **GET** /v1/auth/service-accounts/{id} | Get a service account by ID
[**list_service_accounts**](AuthServiceAccountsApi.md#list_service_accounts) | **GET** /v1/auth/service-accounts | List all service accounts
[**update_service_account**](AuthServiceAccountsApi.md#update_service_account) | **PUT** /v1/auth/service-accounts/{id} | Update a service account


# **create_service_account**
> ServiceAccountResponse create_service_account(create_service_account_request)

Create a new service account

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_service_account_request import CreateServiceAccountRequest
from tachyon_sdk.models.service_account_response import ServiceAccountResponse
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
    api_instance = tachyon_sdk.AuthServiceAccountsApi(api_client)
    create_service_account_request = tachyon_sdk.CreateServiceAccountRequest() # CreateServiceAccountRequest | 

    try:
        # Create a new service account
        api_response = api_instance.create_service_account(create_service_account_request)
        print("The response of AuthServiceAccountsApi->create_service_account:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthServiceAccountsApi->create_service_account: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_service_account_request** | [**CreateServiceAccountRequest**](CreateServiceAccountRequest.md)|  | 

### Return type

[**ServiceAccountResponse**](ServiceAccountResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | Service account created |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_service_account**
> DeleteServiceAccountResponse delete_service_account(id)

Delete a service account

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.delete_service_account_response import DeleteServiceAccountResponse
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
    api_instance = tachyon_sdk.AuthServiceAccountsApi(api_client)
    id = 'id_example' # str | Service account ID

    try:
        # Delete a service account
        api_response = api_instance.delete_service_account(id)
        print("The response of AuthServiceAccountsApi->delete_service_account:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthServiceAccountsApi->delete_service_account: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Service account ID | 

### Return type

[**DeleteServiceAccountResponse**](DeleteServiceAccountResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Service account deleted |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_service_account**
> ServiceAccountResponse get_service_account(id, operator_id)

Get a service account by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.service_account_response import ServiceAccountResponse
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
    api_instance = tachyon_sdk.AuthServiceAccountsApi(api_client)
    id = 'id_example' # str | Service account ID
    operator_id = 'operator_id_example' # str | Operator ID

    try:
        # Get a service account by ID
        api_response = api_instance.get_service_account(id, operator_id)
        print("The response of AuthServiceAccountsApi->get_service_account:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthServiceAccountsApi->get_service_account: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Service account ID | 
 **operator_id** | **str**| Operator ID | 

### Return type

[**ServiceAccountResponse**](ServiceAccountResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Service account found |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_service_accounts**
> ServiceAccountListResponse list_service_accounts(operator_id)

List all service accounts

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.service_account_list_response import ServiceAccountListResponse
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
    api_instance = tachyon_sdk.AuthServiceAccountsApi(api_client)
    operator_id = 'operator_id_example' # str | Operator ID

    try:
        # List all service accounts
        api_response = api_instance.list_service_accounts(operator_id)
        print("The response of AuthServiceAccountsApi->list_service_accounts:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthServiceAccountsApi->list_service_accounts: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **operator_id** | **str**| Operator ID | 

### Return type

[**ServiceAccountListResponse**](ServiceAccountListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Service account list |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_service_account**
> ServiceAccountResponse update_service_account(id, update_service_account_request)

Update a service account

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.service_account_response import ServiceAccountResponse
from tachyon_sdk.models.update_service_account_request import UpdateServiceAccountRequest
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
    api_instance = tachyon_sdk.AuthServiceAccountsApi(api_client)
    id = 'id_example' # str | Service account ID
    update_service_account_request = tachyon_sdk.UpdateServiceAccountRequest() # UpdateServiceAccountRequest | 

    try:
        # Update a service account
        api_response = api_instance.update_service_account(id, update_service_account_request)
        print("The response of AuthServiceAccountsApi->update_service_account:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthServiceAccountsApi->update_service_account: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Service account ID | 
 **update_service_account_request** | [**UpdateServiceAccountRequest**](UpdateServiceAccountRequest.md)|  | 

### Return type

[**ServiceAccountResponse**](ServiceAccountResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Service account updated |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

