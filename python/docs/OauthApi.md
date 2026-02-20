# tachyon_sdk.OauthApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**callback**](OauthApi.md#callback) | **GET** /oauth/{provider_name}/callback | OAuth callback handler for specified provider
[**connect**](OauthApi.md#connect) | **GET** /oauth/{provider_name}/connect | Get OAuth authorization URL for specified provider


# **callback**
> OAuthCallbackResponse callback(provider_name, code, state)

OAuth callback handler for specified provider

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.o_auth_callback_response import OAuthCallbackResponse
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
    api_instance = tachyon_sdk.OauthApi(api_client)
    provider_name = 'provider_name_example' # str | OAuth provider name
    code = 'code_example' # str | Authorization code from provider
    state = 'state_example' # str | State parameter for CSRF protection

    try:
        # OAuth callback handler for specified provider
        api_response = api_instance.callback(provider_name, code, state)
        print("The response of OauthApi->callback:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OauthApi->callback: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **provider_name** | **str**| OAuth provider name | 
 **code** | **str**| Authorization code from provider | 
 **state** | **str**| State parameter for CSRF protection | 

### Return type

[**OAuthCallbackResponse**](OAuthCallbackResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Successfully connected to provider |  -  |
**400** | Failed to exchange code for token |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **connect**
> AuthUrlResponse connect(provider_name)

Get OAuth authorization URL for specified provider

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.auth_url_response import AuthUrlResponse
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
    api_instance = tachyon_sdk.OauthApi(api_client)
    provider_name = 'provider_name_example' # str | OAuth provider name

    try:
        # Get OAuth authorization URL for specified provider
        api_response = api_instance.connect(provider_name)
        print("The response of OauthApi->connect:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OauthApi->connect: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **provider_name** | **str**| OAuth provider name | 

### Return type

[**AuthUrlResponse**](AuthUrlResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Successfully generated authorization URL |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

