# tachyon_sdk.AuthAPIKeysApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_api_key**](AuthAPIKeysApi.md#create_api_key) | **POST** /v1/auth/service-accounts/{service_account_id}/api-keys | Create a new API key for a service account
[**list_api_keys**](AuthAPIKeysApi.md#list_api_keys) | **GET** /v1/auth/service-accounts/{service_account_id}/api-keys | List API keys for a service account


# **create_api_key**
> ApiKeyResponse create_api_key(service_account_id, create_api_key_request)

Create a new API key for a service account

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.api_key_response import ApiKeyResponse
from tachyon_sdk.models.create_api_key_request import CreateApiKeyRequest
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
    api_instance = tachyon_sdk.AuthAPIKeysApi(api_client)
    service_account_id = 'service_account_id_example' # str | Service account ID
    create_api_key_request = tachyon_sdk.CreateApiKeyRequest() # CreateApiKeyRequest | 

    try:
        # Create a new API key for a service account
        api_response = api_instance.create_api_key(service_account_id, create_api_key_request)
        print("The response of AuthAPIKeysApi->create_api_key:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthAPIKeysApi->create_api_key: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **service_account_id** | **str**| Service account ID | 
 **create_api_key_request** | [**CreateApiKeyRequest**](CreateApiKeyRequest.md)|  | 

### Return type

[**ApiKeyResponse**](ApiKeyResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | API key created |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_api_keys**
> ApiKeyListResponse list_api_keys(service_account_id, operator_id)

List API keys for a service account

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.api_key_list_response import ApiKeyListResponse
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
    api_instance = tachyon_sdk.AuthAPIKeysApi(api_client)
    service_account_id = 'service_account_id_example' # str | Service account ID
    operator_id = 'operator_id_example' # str | Operator ID

    try:
        # List API keys for a service account
        api_response = api_instance.list_api_keys(service_account_id, operator_id)
        print("The response of AuthAPIKeysApi->list_api_keys:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthAPIKeysApi->list_api_keys: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **service_account_id** | **str**| Service account ID | 
 **operator_id** | **str**| Operator ID | 

### Return type

[**ApiKeyListResponse**](ApiKeyListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | API key list |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

