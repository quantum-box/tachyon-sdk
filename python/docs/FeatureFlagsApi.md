# tachyon_sdk.FeatureFlagsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**evaluate_actions**](FeatureFlagsApi.md#evaluate_actions) | **POST** /v1/feature-flags/actions/evaluate | TODO: add English documentation


# **evaluate_actions**
> EvaluateActionsResponse evaluate_actions(evaluate_actions_request)

TODO: add English documentation

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.evaluate_actions_request import EvaluateActionsRequest
from tachyon_sdk.models.evaluate_actions_response import EvaluateActionsResponse
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
    api_instance = tachyon_sdk.FeatureFlagsApi(api_client)
    evaluate_actions_request = tachyon_sdk.EvaluateActionsRequest() # EvaluateActionsRequest | 

    try:
        # TODO: add English documentation
        api_response = api_instance.evaluate_actions(evaluate_actions_request)
        print("The response of FeatureFlagsApi->evaluate_actions:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling FeatureFlagsApi->evaluate_actions: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **evaluate_actions_request** | [**EvaluateActionsRequest**](EvaluateActionsRequest.md)|  | 

### Return type

[**EvaluateActionsResponse**](EvaluateActionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | 評価結果 |  -  |
**400** | 不正な入力 |  -  |
**401** | 認証エラー |  -  |
**403** | 権限不足 |  -  |
**500** | 内部エラー |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

