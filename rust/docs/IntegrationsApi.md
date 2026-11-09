# \IntegrationsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**connect_integration**](IntegrationsApi.md#connect_integration) | **POST** /v1/integrations/{id}/connect | Initiate an OAuth connection to an integration.
[**disconnect_integration**](IntegrationsApi.md#disconnect_integration) | **DELETE** /v1/integrations/connections/{id} | Disconnect an integration connection.
[**get_github_billing_usage**](IntegrationsApi.md#get_github_billing_usage) | **GET** /v1/integrations/github/billing-usage | Billing usage of a GitHub organization for one month.
[**get_integration**](IntegrationsApi.md#get_integration) | **GET** /v1/integrations/{id} | Get an integration by ID.
[**list_connections**](IntegrationsApi.md#list_connections) | **GET** /v1/integrations/connections | List all connections for the current tenant.
[**list_integrations**](IntegrationsApi.md#list_integrations) | **GET** /v1/integrations | List all integrations in the marketplace.



## connect_integration

> models::ConnectResponse connect_integration(id)
Initiate an OAuth connection to an integration.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Integration ID | [required] |

### Return type

[**models::ConnectResponse**](ConnectResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## disconnect_integration

> disconnect_integration(id)
Disconnect an integration connection.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Connection ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_github_billing_usage

> serde_json::Value get_github_billing_usage(org, year, month)
Billing usage of a GitHub organization for one month.

The body is GitHub's `GET /organizations/{org}/settings/billing/usage` response, unchanged. The installation token is never returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**org** | **String** | GitHub organization login. It must be the account of a GitHub App installation connected to the requesting tenant. | [required] |
**year** | **i32** |  | [required] |
**month** | **i32** | 1-12. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_integration

> models::IntegrationDetailResponse get_integration(id)
Get an integration by ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Integration ID | [required] |

### Return type

[**models::IntegrationDetailResponse**](IntegrationDetailResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_connections

> models::ListConnectionsResponse list_connections()
List all connections for the current tenant.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListConnectionsResponse**](ListConnectionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_integrations

> models::ListIntegrationsResponse list_integrations(category)
List all integrations in the marketplace.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**category** | Option<**String**> | Filter by category |  |

### Return type

[**models::ListIntegrationsResponse**](ListIntegrationsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

