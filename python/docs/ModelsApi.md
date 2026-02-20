# tachyon_sdk.ModelsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_models**](ModelsApi.md#get_models) | **GET** /v1/llms/models | Get list of supported models


# **get_models**
> ModelsResponse get_models(x_operator_id, authorization, supported_feature=supported_feature, require_agent_product=require_agent_product)

Get list of supported models

Get list of all LLM models available for agents.

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.models_response import ModelsResponse
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
    api_instance = tachyon_sdk.ModelsApi(api_client)
    x_operator_id = 'tn_xxxxxx' # str | Operator ID
    authorization = 'Bearer xxxxx' # str | Authorization
    supported_feature = ['supported_feature_example'] # List[str] | TODO: add English documentation (optional)
    require_agent_product = True # bool | TODO: add English documentation (optional)

    try:
        # Get list of supported models
        api_response = api_instance.get_models(x_operator_id, authorization, supported_feature=supported_feature, require_agent_product=require_agent_product)
        print("The response of ModelsApi->get_models:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ModelsApi->get_models: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **x_operator_id** | **str**| Operator ID | 
 **authorization** | **str**| Authorization | 
 **supported_feature** | [**List[str]**](str.md)| TODO: add English documentation | [optional] 
 **require_agent_product** | **bool**| TODO: add English documentation | [optional] 

### Return type

[**ModelsResponse**](ModelsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | モデル一覧の取得に成功 |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

