# tachyon_sdk.ModelsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_models**](ModelsApi.md#get_models) | **GET** /v1/llms/models | List available LLM models


# **get_models**
> ModelsResponse get_models(x_operator_id, authorization, supported_feature=supported_feature, require_agent_product=require_agent_product)

List available LLM models

Returns all LLM models available for the operator.
Models can be filtered by supported features such as
`streaming`, `function_calling`, `vision`, or `agent`.
By default, only agent-capable models are returned
(`require_agent_product=true`).

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.models_response import ModelsResponse
from tachyon_sdk.models.supported_feature import SupportedFeature
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
    x_operator_id = 'tn_01hjryxysgey07h5jz5wagqj0m' # str | Operator ID
    authorization = 'Bearer dummy-token' # str | Bearer token for authentication
    supported_feature = [tachyon_sdk.SupportedFeature()] # List[SupportedFeature] | Filter models by supported feature. Accepts a single value or comma-separated list (e.g. `?supported_feature=streaming,agent`). When multiple features are specified, only models supporting **all** of them are returned. (optional)
    require_agent_product = true # bool | When `true` (default), only models that support the `agent` feature are returned. The `agent` filter is automatically appended if not already present in `supported_feature`. (optional)

    try:
        # List available LLM models
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
 **authorization** | **str**| Bearer token for authentication | 
 **supported_feature** | [**List[SupportedFeature]**](SupportedFeature.md)| Filter models by supported feature. Accepts a single value or comma-separated list (e.g. &#x60;?supported_feature&#x3D;streaming,agent&#x60;). When multiple features are specified, only models supporting **all** of them are returned. | [optional] 
 **require_agent_product** | **bool**| When &#x60;true&#x60; (default), only models that support the &#x60;agent&#x60; feature are returned. The &#x60;agent&#x60; filter is automatically appended if not already present in &#x60;supported_feature&#x60;. | [optional] 

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
**200** | Successfully retrieved model list |  -  |
**401** | Unauthorized — missing or invalid Authorization header |  -  |
**403** | Forbidden — operator does not have permission |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

