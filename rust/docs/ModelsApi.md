# \ModelsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_models**](ModelsApi.md#get_models) | **GET** /v1/llms/models | List available LLM models



## get_models

> models::ModelsResponse get_models(x_operator_id, authorization, supported_feature, require_agent_product)
List available LLM models

Returns all LLM models available for the operator. Models can be filtered by supported features such as `streaming`, `function_calling`, `vision`, or `agent`. By default, only agent-capable models are returned (`require_agent_product=true`).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_operator_id** | **String** | Operator ID | [required] |
**authorization** | **String** | Bearer token for authentication | [required] |
**supported_feature** | Option<[**Vec<models::SupportedFeature>**](Models__SupportedFeature.md)> | Filter models by supported feature. Accepts a single value or comma-separated list (e.g. `?supported_feature=streaming,agent`). When multiple features are specified, only models supporting **all** of them are returned. |  |
**require_agent_product** | Option<**bool**> | When `true` (default), only models that support the `agent` feature are returned. The `agent` filter is automatically appended if not already present in `supported_feature`. |  |

### Return type

[**models::ModelsResponse**](ModelsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

