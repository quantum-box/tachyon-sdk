# tachyon_sdk.OrderClientsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_client**](OrderClientsApi.md#create_client) | **POST** /v1/order/clients | Create a new client
[**get_client**](OrderClientsApi.md#get_client) | **GET** /v1/order/clients/{id} | Get a client by ID
[**list_clients**](OrderClientsApi.md#list_clients) | **GET** /v1/order/clients | List all clients


# **create_client**
> ClientResponse create_client(create_client_request)

Create a new client

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.client_response import ClientResponse
from tachyon_sdk.models.create_client_request import CreateClientRequest
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
    api_instance = tachyon_sdk.OrderClientsApi(api_client)
    create_client_request = tachyon_sdk.CreateClientRequest() # CreateClientRequest | 

    try:
        # Create a new client
        api_response = api_instance.create_client(create_client_request)
        print("The response of OrderClientsApi->create_client:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderClientsApi->create_client: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_client_request** | [**CreateClientRequest**](CreateClientRequest.md)|  | 

### Return type

[**ClientResponse**](ClientResponse.md)

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
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_client**
> ClientResponse get_client(id)

Get a client by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.client_response import ClientResponse
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
    api_instance = tachyon_sdk.OrderClientsApi(api_client)
    id = 'id_example' # str | Client ID

    try:
        # Get a client by ID
        api_response = api_instance.get_client(id)
        print("The response of OrderClientsApi->get_client:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderClientsApi->get_client: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Client ID | 

### Return type

[**ClientResponse**](ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Client found |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_clients**
> ClientListResponse list_clients()

List all clients

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.client_list_response import ClientListResponse
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
    api_instance = tachyon_sdk.OrderClientsApi(api_client)

    try:
        # List all clients
        api_response = api_instance.list_clients()
        print("The response of OrderClientsApi->list_clients:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderClientsApi->list_clients: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**ClientListResponse**](ClientListResponse.md)

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
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

