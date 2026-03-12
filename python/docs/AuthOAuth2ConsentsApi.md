# tachyon_sdk.AuthOAuth2ConsentsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_consents**](AuthOAuth2ConsentsApi.md#list_consents) | **GET** /v1/auth/consents | GET /v1/auth/consents
[**revoke_consent**](AuthOAuth2ConsentsApi.md#revoke_consent) | **POST** /v1/auth/consents/{id}/revoke | POST /v1/auth/consents/:id/revoke


# **list_consents**
> ConsentListResponse list_consents()

GET /v1/auth/consents

List all consents for the authenticated user.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.consent_list_response import ConsentListResponse
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
    api_instance = tachyon_sdk.AuthOAuth2ConsentsApi(api_client)

    try:
        # GET /v1/auth/consents
        api_response = api_instance.list_consents()
        print("The response of AuthOAuth2ConsentsApi->list_consents:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOAuth2ConsentsApi->list_consents: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**ConsentListResponse**](ConsentListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Consent list |  -  |
**401** | Unauthorized |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **revoke_consent**
> revoke_consent(id)

POST /v1/auth/consents/:id/revoke

Revoke a user consent by ID.

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
    api_instance = tachyon_sdk.AuthOAuth2ConsentsApi(api_client)
    id = 'id_example' # str | Consent ID

    try:
        # POST /v1/auth/consents/:id/revoke
        api_instance.revoke_consent(id)
    except Exception as e:
        print("Exception when calling AuthOAuth2ConsentsApi->revoke_consent: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Consent ID | 

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
**204** | Consent revoked |  -  |
**401** | Unauthorized |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

