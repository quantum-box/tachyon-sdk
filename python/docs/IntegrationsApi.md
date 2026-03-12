# tachyon_sdk.IntegrationsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**connect_integration**](IntegrationsApi.md#connect_integration) | **POST** /v1/integrations/{id}/connect | Initiate an OAuth connection to an integration.
[**disconnect_integration**](IntegrationsApi.md#disconnect_integration) | **DELETE** /v1/integrations/connections/{id} | Disconnect an integration connection.
[**get_integration**](IntegrationsApi.md#get_integration) | **GET** /v1/integrations/{id} | Get an integration by ID.
[**list_connections**](IntegrationsApi.md#list_connections) | **GET** /v1/integrations/connections | List all connections for the current tenant.
[**list_integrations**](IntegrationsApi.md#list_integrations) | **GET** /v1/integrations | List all integrations in the marketplace.


# **connect_integration**
> ConnectResponse connect_integration(id)

Initiate an OAuth connection to an integration.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.connect_response import ConnectResponse
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
    api_instance = tachyon_sdk.IntegrationsApi(api_client)
    id = 'id_example' # str | Integration ID

    try:
        # Initiate an OAuth connection to an integration.
        api_response = api_instance.connect_integration(id)
        print("The response of IntegrationsApi->connect_integration:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IntegrationsApi->connect_integration: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Integration ID | 

### Return type

[**ConnectResponse**](ConnectResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | OAuth authorization URL |  -  |
**403** | Forbidden |  -  |
**404** | Integration not found |  -  |
**409** | Already connected |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **disconnect_integration**
> disconnect_integration(id)

Disconnect an integration connection.

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
    api_instance = tachyon_sdk.IntegrationsApi(api_client)
    id = 'id_example' # str | Connection ID

    try:
        # Disconnect an integration connection.
        api_instance.disconnect_integration(id)
    except Exception as e:
        print("Exception when calling IntegrationsApi->disconnect_integration: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Connection ID | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Disconnected |  -  |
**403** | Forbidden |  -  |
**404** | Connection not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_integration**
> IntegrationDetailResponse get_integration(id)

Get an integration by ID.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.integration_detail_response import IntegrationDetailResponse
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
    api_instance = tachyon_sdk.IntegrationsApi(api_client)
    id = 'id_example' # str | Integration ID

    try:
        # Get an integration by ID.
        api_response = api_instance.get_integration(id)
        print("The response of IntegrationsApi->get_integration:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IntegrationsApi->get_integration: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Integration ID | 

### Return type

[**IntegrationDetailResponse**](IntegrationDetailResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Integration details |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_connections**
> ListConnectionsResponse list_connections()

List all connections for the current tenant.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.list_connections_response import ListConnectionsResponse
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
    api_instance = tachyon_sdk.IntegrationsApi(api_client)

    try:
        # List all connections for the current tenant.
        api_response = api_instance.list_connections()
        print("The response of IntegrationsApi->list_connections:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IntegrationsApi->list_connections: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**ListConnectionsResponse**](ListConnectionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Connection list |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_integrations**
> ListIntegrationsResponse list_integrations(category=category)

List all integrations in the marketplace.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.list_integrations_response import ListIntegrationsResponse
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
    api_instance = tachyon_sdk.IntegrationsApi(api_client)
    category = 'category_example' # str | Filter by category (optional)

    try:
        # List all integrations in the marketplace.
        api_response = api_instance.list_integrations(category=category)
        print("The response of IntegrationsApi->list_integrations:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling IntegrationsApi->list_integrations: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **category** | **str**| Filter by category | [optional] 

### Return type

[**ListIntegrationsResponse**](ListIntegrationsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Integration list |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

