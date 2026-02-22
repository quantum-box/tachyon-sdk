# tachyon_sdk.AuthOAuthTokensApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_oauth_token**](AuthOAuthTokensApi.md#delete_oauth_token) | **DELETE** /v1/auth/oauth-tokens/{provider} | Delete an OAuth token
[**get_oauth_token_by_provider**](AuthOAuthTokensApi.md#get_oauth_token_by_provider) | **GET** /v1/auth/oauth-tokens/{provider} | Get an OAuth token by provider
[**list_oauth_tokens**](AuthOAuthTokensApi.md#list_oauth_tokens) | **GET** /v1/auth/oauth-tokens | List all OAuth tokens
[**save_oauth_token**](AuthOAuthTokensApi.md#save_oauth_token) | **POST** /v1/auth/oauth-tokens | Save an OAuth token


# **delete_oauth_token**
> delete_oauth_token(provider)

Delete an OAuth token

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
    api_instance = tachyon_sdk.AuthOAuthTokensApi(api_client)
    provider = 'provider_example' # str | OAuth provider name

    try:
        # Delete an OAuth token
        api_instance.delete_oauth_token(provider)
    except Exception as e:
        print("Exception when calling AuthOAuthTokensApi->delete_oauth_token: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **provider** | **str**| OAuth provider name | 

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
**200** | OAuth token deleted |  -  |
**403** | Forbidden |  -  |
**404** | Token not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_oauth_token_by_provider**
> OAuthTokenDetailResponse get_oauth_token_by_provider(provider)

Get an OAuth token by provider

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.o_auth_token_detail_response import OAuthTokenDetailResponse
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
    api_instance = tachyon_sdk.AuthOAuthTokensApi(api_client)
    provider = 'provider_example' # str | OAuth provider name

    try:
        # Get an OAuth token by provider
        api_response = api_instance.get_oauth_token_by_provider(provider)
        print("The response of AuthOAuthTokensApi->get_oauth_token_by_provider:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuthTokensApi->get_oauth_token_by_provider: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **provider** | **str**| OAuth provider name | 

### Return type

[**OAuthTokenDetailResponse**](OAuthTokenDetailResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | OAuth token found |  -  |
**404** | Token not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_oauth_tokens**
> OAuthTokenListResponse list_oauth_tokens()

List all OAuth tokens

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.o_auth_token_list_response import OAuthTokenListResponse
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
    api_instance = tachyon_sdk.AuthOAuthTokensApi(api_client)

    try:
        # List all OAuth tokens
        api_response = api_instance.list_oauth_tokens()
        print("The response of AuthOAuthTokensApi->list_oauth_tokens:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuthTokensApi->list_oauth_tokens: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**OAuthTokenListResponse**](OAuthTokenListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | OAuth token list |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **save_oauth_token**
> OAuthTokenResponse save_oauth_token(save_o_auth_token_request)

Save an OAuth token

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.o_auth_token_response import OAuthTokenResponse
from tachyon_sdk.models.save_o_auth_token_request import SaveOAuthTokenRequest
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
    api_instance = tachyon_sdk.AuthOAuthTokensApi(api_client)
    save_o_auth_token_request = tachyon_sdk.SaveOAuthTokenRequest() # SaveOAuthTokenRequest | 

    try:
        # Save an OAuth token
        api_response = api_instance.save_oauth_token(save_o_auth_token_request)
        print("The response of AuthOAuthTokensApi->save_oauth_token:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuthTokensApi->save_oauth_token: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **save_o_auth_token_request** | [**SaveOAuthTokenRequest**](SaveOAuthTokenRequest.md)|  | 

### Return type

[**OAuthTokenResponse**](OAuthTokenResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | OAuth token saved |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

