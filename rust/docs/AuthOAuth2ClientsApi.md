# \AuthOAuth2ClientsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_client**](AuthOAuth2ClientsApi.md#create_client) | **POST** /v1/auth/oauth2-clients | Create a new OAuth2 client
[**get_client**](AuthOAuth2ClientsApi.md#get_client) | **GET** /v1/auth/oauth2-clients/{id} | Get an OAuth2 client by ID
[**list_clients**](AuthOAuth2ClientsApi.md#list_clients) | **GET** /v1/auth/oauth2-clients | List all OAuth2 clients for the current tenant
[**revoke_client**](AuthOAuth2ClientsApi.md#revoke_client) | **POST** /v1/auth/oauth2-clients/{id}/revoke | Revoke an OAuth2 client
[**rotate_secret**](AuthOAuth2ClientsApi.md#rotate_secret) | **POST** /v1/auth/oauth2-clients/{id}/rotate-secret | Rotate an OAuth2 client secret
[**update_client**](AuthOAuth2ClientsApi.md#update_client) | **PUT** /v1/auth/oauth2-clients/{id} | Update an OAuth2 client



## create_client

> models::CreateClientResponse create_client(create_client_request)
Create a new OAuth2 client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_client_request** | [**CreateClientRequest**](CreateClientRequest.md) |  | [required] |

### Return type

[**models::CreateClientResponse**](CreateClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client

> models::ClientResponse get_client(id)
Get an OAuth2 client by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | OAuth2 client ID | [required] |

### Return type

[**models::ClientResponse**](ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_clients

> models::ClientListResponse list_clients()
List all OAuth2 clients for the current tenant

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ClientListResponse**](ClientListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_client

> revoke_client(id)
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


## rotate_secret

> models::RotateSecretResponse rotate_secret(id)
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


## update_client

> models::ClientResponse update_client(id, update_client_request)
Update an OAuth2 client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | OAuth2 client ID | [required] |
**update_client_request** | [**UpdateClientRequest**](UpdateClientRequest.md) |  | [required] |

### Return type

[**models::ClientResponse**](ClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

