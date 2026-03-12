# \AuthOAuth2ConsentsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_consents**](AuthOAuth2ConsentsApi.md#list_consents) | **GET** /v1/auth/consents | GET /v1/auth/consents
[**revoke_consent**](AuthOAuth2ConsentsApi.md#revoke_consent) | **POST** /v1/auth/consents/{id}/revoke | POST /v1/auth/consents/:id/revoke



## list_consents

> models::ConsentListResponse list_consents()
GET /v1/auth/consents

List all consents for the authenticated user.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ConsentListResponse**](ConsentListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_consent

> revoke_consent(id)
POST /v1/auth/consents/:id/revoke

Revoke a user consent by ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Consent ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

