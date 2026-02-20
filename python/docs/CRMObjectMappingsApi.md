# tachyon_sdk.CRMObjectMappingsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_object_mapping**](CRMObjectMappingsApi.md#create_object_mapping) | **POST** /v1/crm/object-mappings | Create an object mapping
[**get_object_mappings**](CRMObjectMappingsApi.md#get_object_mappings) | **GET** /v1/crm/object-mappings | Get object mappings by entity ID and object name


# **create_object_mapping**
> ObjectMappingResponse create_object_mapping(create_object_mapping_request)

Create an object mapping

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_object_mapping_request import CreateObjectMappingRequest
from tachyon_sdk.models.object_mapping_response import ObjectMappingResponse
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
    api_instance = tachyon_sdk.CRMObjectMappingsApi(api_client)
    create_object_mapping_request = tachyon_sdk.CreateObjectMappingRequest() # CreateObjectMappingRequest | 

    try:
        # Create an object mapping
        api_response = api_instance.create_object_mapping(create_object_mapping_request)
        print("The response of CRMObjectMappingsApi->create_object_mapping:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling CRMObjectMappingsApi->create_object_mapping: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_object_mapping_request** | [**CreateObjectMappingRequest**](CreateObjectMappingRequest.md)|  | 

### Return type

[**ObjectMappingResponse**](ObjectMappingResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | Object mapping created |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_object_mappings**
> ObjectMappingListResponse get_object_mappings(entity_id, object_name)

Get object mappings by entity ID and object name

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.object_mapping_list_response import ObjectMappingListResponse
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
    api_instance = tachyon_sdk.CRMObjectMappingsApi(api_client)
    entity_id = 'entity_id_example' # str | Entity ID to look up
    object_name = 'object_name_example' # str | Object type (Deal, Product, etc.)

    try:
        # Get object mappings by entity ID and object name
        api_response = api_instance.get_object_mappings(entity_id, object_name)
        print("The response of CRMObjectMappingsApi->get_object_mappings:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling CRMObjectMappingsApi->get_object_mappings: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **entity_id** | **str**| Entity ID to look up | 
 **object_name** | **str**| Object type (Deal, Product, etc.) | 

### Return type

[**ObjectMappingListResponse**](ObjectMappingListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Object mapping list |  -  |
**400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

