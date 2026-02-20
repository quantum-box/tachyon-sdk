# tachyon_sdk.AuthVerifyApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**sign_in_with_platform**](AuthVerifyApi.md#sign_in_with_platform) | **POST** /auth/v1beta/sign-in-with-platform | 
[**verify**](AuthVerifyApi.md#verify) | **POST** /auth/v1beta/verify | 


# **sign_in_with_platform**
> SignInWithPlatformResponse sign_in_with_platform(sign_in_with_platform_request)

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.sign_in_with_platform_request import SignInWithPlatformRequest
from tachyon_sdk.models.sign_in_with_platform_response import SignInWithPlatformResponse
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
    api_instance = tachyon_sdk.AuthVerifyApi(api_client)
    sign_in_with_platform_request = tachyon_sdk.SignInWithPlatformRequest() # SignInWithPlatformRequest | 

    try:
        api_response = api_instance.sign_in_with_platform(sign_in_with_platform_request)
        print("The response of AuthVerifyApi->sign_in_with_platform:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthVerifyApi->sign_in_with_platform: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **sign_in_with_platform_request** | [**SignInWithPlatformRequest**](SignInWithPlatformRequest.md)|  | 

### Return type

[**SignInWithPlatformResponse**](SignInWithPlatformResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User signed in or created |  -  |
**400** | Bad request |  -  |
**401** | Unauthorized |  -  |
**500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **verify**
> VerifyResponse verify(verify_request)

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.verify_request import VerifyRequest
from tachyon_sdk.models.verify_response import VerifyResponse
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
    api_instance = tachyon_sdk.AuthVerifyApi(api_client)
    verify_request = tachyon_sdk.VerifyRequest() # VerifyRequest | 

    try:
        api_response = api_instance.verify(verify_request)
        print("The response of AuthVerifyApi->verify:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthVerifyApi->verify: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **verify_request** | [**VerifyRequest**](VerifyRequest.md)|  | 

### Return type

[**VerifyResponse**](VerifyResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Successful response |  -  |
**401** | Unauthorized |  -  |
**403** | Forbidden |  -  |
**500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

