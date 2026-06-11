# tachyon_sdk.AuthOAuth2ClientsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_oauth2_client**](AuthOAuth2ClientsApi.md#create_oauth2_client) | **POST** /v1/auth/oauth2-clients | Create a new OAuth2 client
[**get_oauth2_client**](AuthOAuth2ClientsApi.md#get_oauth2_client) | **GET** /v1/auth/oauth2-clients/{id} | Get an OAuth2 client by ID
[**list_oauth2_clients**](AuthOAuth2ClientsApi.md#list_oauth2_clients) | **GET** /v1/auth/oauth2-clients | List all OAuth2 clients for the current tenant
[**revoke_oauth2_client**](AuthOAuth2ClientsApi.md#revoke_oauth2_client) | **POST** /v1/auth/oauth2-clients/{id}/revoke | Revoke an OAuth2 client
[**rotate_oauth2_client_secret**](AuthOAuth2ClientsApi.md#rotate_oauth2_client_secret) | **POST** /v1/auth/oauth2-clients/{id}/rotate-secret | Rotate an OAuth2 client secret
[**update_oauth2_client**](AuthOAuth2ClientsApi.md#update_oauth2_client) | **PUT** /v1/auth/oauth2-clients/{id} | Update an OAuth2 client


# **create_oauth2_client**
> OAuth2CreateClientResponse create_oauth2_client(o_auth2_create_client_request)

Create a new OAuth2 client

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.o_auth2_create_client_request import OAuth2CreateClientRequest
from tachyon_sdk.models.o_auth2_create_client_response import OAuth2CreateClientResponse
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
    api_instance = tachyon_sdk.AuthOAuth2ClientsApi(api_client)
    o_auth2_create_client_request = tachyon_sdk.OAuth2CreateClientRequest() # OAuth2CreateClientRequest |

    try:
        # Create a new OAuth2 client
        api_response = api_instance.create_oauth2_client(o_auth2_create_client_request)
        print("The response of AuthOAuth2ClientsApi->create_oauth2_client:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuth2ClientsApi->create_oauth2_client: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **o_auth2_create_client_request** | [**OAuth2CreateClientRequest**](OAuth2CreateClientRequest.md)|  |

### Return type

[**OAuth2CreateClientResponse**](OAuth2CreateClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | Client created |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_oauth2_client**
> OAuth2ClientResponse get_oauth2_client(id)

Get an OAuth2 client by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.o_auth2_client_response import OAuth2ClientResponse
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
    api_instance = tachyon_sdk.AuthOAuth2ClientsApi(api_client)
    id = 'id_example' # str | OAuth2 client ID

    try:
        # Get an OAuth2 client by ID
        api_response = api_instance.get_oauth2_client(id)
        print("The response of AuthOAuth2ClientsApi->get_oauth2_client:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuth2ClientsApi->get_oauth2_client: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| OAuth2 client ID |

### Return type

[**OAuth2ClientResponse**](OAuth2ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Client found |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_oauth2_clients**
> OAuth2ClientListResponse list_oauth2_clients()

List all OAuth2 clients for the current tenant

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.o_auth2_client_list_response import OAuth2ClientListResponse
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
    api_instance = tachyon_sdk.AuthOAuth2ClientsApi(api_client)

    try:
        # List all OAuth2 clients for the current tenant
        api_response = api_instance.list_oauth2_clients()
        print("The response of AuthOAuth2ClientsApi->list_oauth2_clients:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuth2ClientsApi->list_oauth2_clients: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**OAuth2ClientListResponse**](OAuth2ClientListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Client list |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **revoke_oauth2_client**
> revoke_oauth2_client(id)

Revoke an OAuth2 client

### Example


```python
import tachyon_sdk
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
    api_instance = tachyon_sdk.AuthOAuth2ClientsApi(api_client)
    id = 'id_example' # str | OAuth2 client ID

    try:
        # Revoke an OAuth2 client
        api_instance.revoke_oauth2_client(id)
    except Exception as e:
        print("Exception when calling AuthOAuth2ClientsApi->revoke_oauth2_client: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| OAuth2 client ID |

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Client revoked |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **rotate_oauth2_client_secret**
> RotateSecretResponse rotate_oauth2_client_secret(id)

Rotate an OAuth2 client secret

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.rotate_secret_response import RotateSecretResponse
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
    api_instance = tachyon_sdk.AuthOAuth2ClientsApi(api_client)
    id = 'id_example' # str | OAuth2 client ID

    try:
        # Rotate an OAuth2 client secret
        api_response = api_instance.rotate_oauth2_client_secret(id)
        print("The response of AuthOAuth2ClientsApi->rotate_oauth2_client_secret:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuth2ClientsApi->rotate_oauth2_client_secret: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| OAuth2 client ID |

### Return type

[**RotateSecretResponse**](RotateSecretResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Secret rotated |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_oauth2_client**
> OAuth2ClientResponse update_oauth2_client(id, o_auth2_update_client_request)

Update an OAuth2 client

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.o_auth2_client_response import OAuth2ClientResponse
from tachyon_sdk.models.o_auth2_update_client_request import OAuth2UpdateClientRequest
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
    api_instance = tachyon_sdk.AuthOAuth2ClientsApi(api_client)
    id = 'id_example' # str | OAuth2 client ID
    o_auth2_update_client_request = tachyon_sdk.OAuth2UpdateClientRequest() # OAuth2UpdateClientRequest |

    try:
        # Update an OAuth2 client
        api_response = api_instance.update_oauth2_client(id, o_auth2_update_client_request)
        print("The response of AuthOAuth2ClientsApi->update_oauth2_client:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuth2ClientsApi->update_oauth2_client: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| OAuth2 client ID |
 **o_auth2_update_client_request** | [**OAuth2UpdateClientRequest**](OAuth2UpdateClientRequest.md)|  |

### Return type

[**OAuth2ClientResponse**](OAuth2ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Client updated |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

