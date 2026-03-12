# tachyon_sdk.AuthOperatorsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_operator**](AuthOperatorsApi.md#create_operator) | **POST** /v1/auth/operators | Create an operator under a platform
[**find_operators_by_user**](AuthOperatorsApi.md#find_operators_by_user) | **GET** /v1/auth/operators/by-user | Find operators accessible to a user under a platform
[**get_operator_by_alias**](AuthOperatorsApi.md#get_operator_by_alias) | **GET** /v1/auth/operators/by-alias | Get an operator by alias within a platform
[**get_operator_by_id**](AuthOperatorsApi.md#get_operator_by_id) | **GET** /v1/auth/operators/{id} | Get an operator by ID


# **create_operator**
> CreateOperatorResponse create_operator(create_operator_request)

Create an operator under a platform

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_operator_request import CreateOperatorRequest
from tachyon_sdk.models.create_operator_response import CreateOperatorResponse
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
    api_instance = tachyon_sdk.AuthOperatorsApi(api_client)
    create_operator_request = tachyon_sdk.CreateOperatorRequest() # CreateOperatorRequest | 

    try:
        # Create an operator under a platform
        api_response = api_instance.create_operator(create_operator_request)
        print("The response of AuthOperatorsApi->create_operator:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOperatorsApi->create_operator: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_operator_request** | [**CreateOperatorRequest**](CreateOperatorRequest.md)|  | 

### Return type

[**CreateOperatorResponse**](CreateOperatorResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Operator created |  -  |
**400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **find_operators_by_user**
> OperatorListResponse find_operators_by_user(platform_id, user_id)

Find operators accessible to a user under a platform

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.operator_list_response import OperatorListResponse
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
    api_instance = tachyon_sdk.AuthOperatorsApi(api_client)
    platform_id = 'platform_id_example' # str | Platform ID
    user_id = 'user_id_example' # str | User ID

    try:
        # Find operators accessible to a user under a platform
        api_response = api_instance.find_operators_by_user(platform_id, user_id)
        print("The response of AuthOperatorsApi->find_operators_by_user:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOperatorsApi->find_operators_by_user: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **platform_id** | **str**| Platform ID | 
 **user_id** | **str**| User ID | 

### Return type

[**OperatorListResponse**](OperatorListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Operators found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_operator_by_alias**
> OperatorResponse get_operator_by_alias(platform_id, alias)

Get an operator by alias within a platform

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.operator_response import OperatorResponse
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
    api_instance = tachyon_sdk.AuthOperatorsApi(api_client)
    platform_id = 'platform_id_example' # str | Platform ID
    alias = 'alias_example' # str | Operator alias (username)

    try:
        # Get an operator by alias within a platform
        api_response = api_instance.get_operator_by_alias(platform_id, alias)
        print("The response of AuthOperatorsApi->get_operator_by_alias:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOperatorsApi->get_operator_by_alias: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **platform_id** | **str**| Platform ID | 
 **alias** | **str**| Operator alias (username) | 

### Return type

[**OperatorResponse**](OperatorResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Operator found |  -  |
**404** | Operator not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_operator_by_id**
> OperatorResponse get_operator_by_id(id)

Get an operator by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.operator_response import OperatorResponse
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
    api_instance = tachyon_sdk.AuthOperatorsApi(api_client)
    id = 'id_example' # str | Operator ID

    try:
        # Get an operator by ID
        api_response = api_instance.get_operator_by_id(id)
        print("The response of AuthOperatorsApi->get_operator_by_id:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthOperatorsApi->get_operator_by_id: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Operator ID | 

### Return type

[**OperatorResponse**](OperatorResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Operator found |  -  |
**404** | Operator not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

