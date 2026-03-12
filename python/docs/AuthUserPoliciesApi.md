# tachyon_sdk.AuthUserPoliciesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**attach_user_policy**](AuthUserPoliciesApi.md#attach_user_policy) | **POST** /v1/auth/user-policies/attach | Attach a policy to a user
[**attach_user_policy_with_scope**](AuthUserPoliciesApi.md#attach_user_policy_with_scope) | **POST** /v1/auth/user-policies/attach-with-scope | Attach a policy to a user with resource scope
[**detach_user_policy**](AuthUserPoliciesApi.md#detach_user_policy) | **POST** /v1/auth/user-policies/detach | Detach a policy from a user
[**detach_user_policy_with_scope**](AuthUserPoliciesApi.md#detach_user_policy_with_scope) | **POST** /v1/auth/user-policies/detach-with-scope | Detach a scoped policy from a user
[**find_user_policy_mappings**](AuthUserPoliciesApi.md#find_user_policy_mappings) | **GET** /v1/auth/user-policy-mappings | Find user policy mappings by resource scope
[**list_user_policies**](AuthUserPoliciesApi.md#list_user_policies) | **GET** /v1/auth/users/{user_id}/policies | List policies attached to a user


# **attach_user_policy**
> attach_user_policy(attach_user_policy_request)

Attach a policy to a user

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.attach_user_policy_request import AttachUserPolicyRequest
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
    api_instance = tachyon_sdk.AuthUserPoliciesApi(api_client)
    attach_user_policy_request = tachyon_sdk.AttachUserPolicyRequest() # AttachUserPolicyRequest | 

    try:
        # Attach a policy to a user
        api_instance.attach_user_policy(attach_user_policy_request)
    except Exception as e:
        print("Exception when calling AuthUserPoliciesApi->attach_user_policy: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **attach_user_policy_request** | [**AttachUserPolicyRequest**](AttachUserPolicyRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: Not defined

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Policy attached |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **attach_user_policy_with_scope**
> attach_user_policy_with_scope(attach_user_policy_with_scope_request)

Attach a policy to a user with resource scope

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.attach_user_policy_with_scope_request import AttachUserPolicyWithScopeRequest
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
    api_instance = tachyon_sdk.AuthUserPoliciesApi(api_client)
    attach_user_policy_with_scope_request = tachyon_sdk.AttachUserPolicyWithScopeRequest() # AttachUserPolicyWithScopeRequest | 

    try:
        # Attach a policy to a user with resource scope
        api_instance.attach_user_policy_with_scope(attach_user_policy_with_scope_request)
    except Exception as e:
        print("Exception when calling AuthUserPoliciesApi->attach_user_policy_with_scope: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **attach_user_policy_with_scope_request** | [**AttachUserPolicyWithScopeRequest**](AttachUserPolicyWithScopeRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: Not defined

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Scoped policy attached |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **detach_user_policy**
> detach_user_policy(detach_user_policy_request)

Detach a policy from a user

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.detach_user_policy_request import DetachUserPolicyRequest
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
    api_instance = tachyon_sdk.AuthUserPoliciesApi(api_client)
    detach_user_policy_request = tachyon_sdk.DetachUserPolicyRequest() # DetachUserPolicyRequest | 

    try:
        # Detach a policy from a user
        api_instance.detach_user_policy(detach_user_policy_request)
    except Exception as e:
        print("Exception when calling AuthUserPoliciesApi->detach_user_policy: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **detach_user_policy_request** | [**DetachUserPolicyRequest**](DetachUserPolicyRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: Not defined

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Policy detached |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **detach_user_policy_with_scope**
> detach_user_policy_with_scope(detach_user_policy_with_scope_request)

Detach a scoped policy from a user

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.detach_user_policy_with_scope_request import DetachUserPolicyWithScopeRequest
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
    api_instance = tachyon_sdk.AuthUserPoliciesApi(api_client)
    detach_user_policy_with_scope_request = tachyon_sdk.DetachUserPolicyWithScopeRequest() # DetachUserPolicyWithScopeRequest | 

    try:
        # Detach a scoped policy from a user
        api_instance.detach_user_policy_with_scope(detach_user_policy_with_scope_request)
    except Exception as e:
        print("Exception when calling AuthUserPoliciesApi->detach_user_policy_with_scope: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **detach_user_policy_with_scope_request** | [**DetachUserPolicyWithScopeRequest**](DetachUserPolicyWithScopeRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: Not defined

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Scoped policy detached |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **find_user_policy_mappings**
> UserPolicyMappingListResponse find_user_policy_mappings(tenant_id, resource_scope)

Find user policy mappings by resource scope

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.user_policy_mapping_list_response import UserPolicyMappingListResponse
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
    api_instance = tachyon_sdk.AuthUserPoliciesApi(api_client)
    tenant_id = 'tenant_id_example' # str | Tenant ID
    resource_scope = 'resource_scope_example' # str | Resource scope in TRN format

    try:
        # Find user policy mappings by resource scope
        api_response = api_instance.find_user_policy_mappings(tenant_id, resource_scope)
        print("The response of AuthUserPoliciesApi->find_user_policy_mappings:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUserPoliciesApi->find_user_policy_mappings: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tenant_id** | **str**| Tenant ID | 
 **resource_scope** | **str**| Resource scope in TRN format | 

### Return type

[**UserPolicyMappingListResponse**](UserPolicyMappingListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User policy mappings |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_user_policies**
> UserPolicyListResponse list_user_policies(user_id, tenant_id=tenant_id)

List policies attached to a user

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.user_policy_list_response import UserPolicyListResponse
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
    api_instance = tachyon_sdk.AuthUserPoliciesApi(api_client)
    user_id = 'user_id_example' # str | User ID
    tenant_id = 'tenant_id_example' # str | Optional tenant ID filter (optional)

    try:
        # List policies attached to a user
        api_response = api_instance.list_user_policies(user_id, tenant_id=tenant_id)
        print("The response of AuthUserPoliciesApi->list_user_policies:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthUserPoliciesApi->list_user_policies: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user_id** | **str**| User ID | 
 **tenant_id** | **str**| Optional tenant ID filter | [optional] 

### Return type

[**UserPolicyListResponse**](UserPolicyListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | User policy list |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

