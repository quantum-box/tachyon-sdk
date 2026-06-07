# \AuthOAuth2ClientsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_oauth2_client**](AuthOAuth2ClientsApi.md#create_oauth2_client) | **POST** /v1/auth/oauth2-clients | Create a new OAuth2 client
[**get_oauth2_client**](AuthOAuth2ClientsApi.md#get_oauth2_client) | **GET** /v1/auth/oauth2-clients/{id} | Get an OAuth2 client by ID
[**list_oauth2_clients**](AuthOAuth2ClientsApi.md#list_oauth2_clients) | **GET** /v1/auth/oauth2-clients | List all OAuth2 clients for the current tenant
[**revoke_oauth2_client**](AuthOAuth2ClientsApi.md#revoke_oauth2_client) | **POST** /v1/auth/oauth2-clients/{id}/revoke | Revoke an OAuth2 client
[**rotate_oauth2_client_secret**](AuthOAuth2ClientsApi.md#rotate_oauth2_client_secret) | **POST** /v1/auth/oauth2-clients/{id}/rotate-secret | Rotate an OAuth2 client secret
[**update_oauth2_client**](AuthOAuth2ClientsApi.md#update_oauth2_client) | **PUT** /v1/auth/oauth2-clients/{id} | Update an OAuth2 client



## create_oauth2_client

> models::OAuth2CreateClientResponse create_oauth2_client(o_auth2_create_client_request)
Create a new OAuth2 client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**o_auth2_create_client_request** | [**OAuth2CreateClientRequest**](OAuth2CreateClientRequest.md) |  | [required] |

### Return type

[**models::OAuth2CreateClientResponse**](OAuth2CreateClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_oauth2_client

> models::OAuth2ClientResponse get_oauth2_client(id)
Get an OAuth2 client by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | OAuth2 client ID | [required] |

### Return type

[**models::OAuth2ClientResponse**](OAuth2ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_oauth2_clients

> models::OAuth2ClientListResponse list_oauth2_clients()
List all OAuth2 clients for the current tenant

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::OAuth2ClientListResponse**](OAuth2ClientListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_oauth2_client

> revoke_oauth2_client(id)
Revoke an OAuth2 client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | OAuth2 client ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## rotate_oauth2_client_secret

> models::RotateSecretResponse rotate_oauth2_client_secret(id)
Rotate an OAuth2 client secret

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | OAuth2 client ID | [required] |

### Return type

[**models::RotateSecretResponse**](RotateSecretResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_oauth2_client

> models::OAuth2ClientResponse update_oauth2_client(id, o_auth2_update_client_request)
Update an OAuth2 client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | OAuth2 client ID | [required] |
**o_auth2_update_client_request** | [**OAuth2UpdateClientRequest**](OAuth2UpdateClientRequest.md) |  | [required] |

### Return type

[**models::OAuth2ClientResponse**](OAuth2ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

