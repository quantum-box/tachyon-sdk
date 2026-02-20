# tachyon_sdk.AuthPoliciesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**evaluate_policies_batch**](AuthPoliciesApi.md#evaluate_policies_batch) | **POST** /v1/auth/policies/check | Evaluate multiple policy actions in batch
[**get_policy**](AuthPoliciesApi.md#get_policy) | **GET** /v1/auth/policies/{id} | Get a policy by ID
[**list_actions**](AuthPoliciesApi.md#list_actions) | **GET** /v1/auth/actions | List all registered actions


# **evaluate_policies_batch**
> EvaluatePoliciesBatchResponse evaluate_policies_batch(evaluate_policies_batch_request)

Evaluate multiple policy actions in batch

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.evaluate_policies_batch_request import EvaluatePoliciesBatchRequest
from tachyon_sdk.models.evaluate_policies_batch_response import EvaluatePoliciesBatchResponse
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
    api_instance = tachyon_sdk.AuthPoliciesApi(api_client)
    evaluate_policies_batch_request = tachyon_sdk.EvaluatePoliciesBatchRequest() # EvaluatePoliciesBatchRequest | 

    try:
        # Evaluate multiple policy actions in batch
        api_response = api_instance.evaluate_policies_batch(evaluate_policies_batch_request)
        print("The response of AuthPoliciesApi->evaluate_policies_batch:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthPoliciesApi->evaluate_policies_batch: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **evaluate_policies_batch_request** | [**EvaluatePoliciesBatchRequest**](EvaluatePoliciesBatchRequest.md)|  | 

### Return type

[**EvaluatePoliciesBatchResponse**](EvaluatePoliciesBatchResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Evaluation results |  -  |
**400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_policy**
> PolicyResponse get_policy(id)

Get a policy by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.policy_response import PolicyResponse
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
    api_instance = tachyon_sdk.AuthPoliciesApi(api_client)
    id = 'id_example' # str | Policy ID

    try:
        # Get a policy by ID
        api_response = api_instance.get_policy(id)
        print("The response of AuthPoliciesApi->get_policy:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthPoliciesApi->get_policy: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Policy ID | 

### Return type

[**PolicyResponse**](PolicyResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Policy found |  -  |
**404** | Policy not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_actions**
> ActionListResponse list_actions(context=context)

List all registered actions

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.action_list_response import ActionListResponse
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
    api_instance = tachyon_sdk.AuthPoliciesApi(api_client)
    context = 'context_example' # str | Filter by context (optional)

    try:
        # List all registered actions
        api_response = api_instance.list_actions(context=context)
        print("The response of AuthPoliciesApi->list_actions:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthPoliciesApi->list_actions: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **context** | **str**| Filter by context | [optional] 

### Return type

[**ActionListResponse**](ActionListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Action list |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

