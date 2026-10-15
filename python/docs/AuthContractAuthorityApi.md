# tachyon_sdk.AuthContractAuthorityApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**resolve**](AuthContractAuthorityApi.md#resolve) | **POST** /v1/auth/contract-authority/resolve | 


# **resolve**
> ContractAuthorityResolveResponse resolve(contract_authority_resolve_request)

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.contract_authority_resolve_request import ContractAuthorityResolveRequest
from tachyon_sdk.models.contract_authority_resolve_response import ContractAuthorityResolveResponse
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
    api_instance = tachyon_sdk.AuthContractAuthorityApi(api_client)
    contract_authority_resolve_request = tachyon_sdk.ContractAuthorityResolveRequest() # ContractAuthorityResolveRequest | 

    try:
        api_response = api_instance.resolve(contract_authority_resolve_request)
        print("The response of AuthContractAuthorityApi->resolve:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthContractAuthorityApi->resolve: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **contract_authority_resolve_request** | [**ContractAuthorityResolveRequest**](ContractAuthorityResolveRequest.md)|  | 

### Return type

[**ContractAuthorityResolveResponse**](ContractAuthorityResolveResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Current verified authority for the exact binding slot |  -  |
**400** | Invalid request scope or shape |  -  |
**403** | Caller denied or current proof unavailable |  -  |
**503** | Authority source unavailable |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

