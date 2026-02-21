# tachyon_sdk.ProcurementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_variant_link**](ProcurementApi.md#delete_variant_link) | **DELETE** /v1/procurement/variant-links/{id} | Delete a variant procurement link
[**upsert_variant_link**](ProcurementApi.md#upsert_variant_link) | **POST** /v1/procurement/variant-links | Upsert a variant procurement link


# **delete_variant_link**
> DeleteResponse delete_variant_link(id)

Delete a variant procurement link

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.delete_response import DeleteResponse
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
    api_instance = tachyon_sdk.ProcurementApi(api_client)
    id = 'id_example' # str | Link ID

    try:
        # Delete a variant procurement link
        api_response = api_instance.delete_variant_link(id)
        print("The response of ProcurementApi->delete_variant_link:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ProcurementApi->delete_variant_link: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Link ID | 

### Return type

[**DeleteResponse**](DeleteResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Link deleted |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **upsert_variant_link**
> VariantLinkResponse upsert_variant_link(upsert_variant_link_request)

Upsert a variant procurement link

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.upsert_variant_link_request import UpsertVariantLinkRequest
from tachyon_sdk.models.variant_link_response import VariantLinkResponse
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
    api_instance = tachyon_sdk.ProcurementApi(api_client)
    upsert_variant_link_request = tachyon_sdk.UpsertVariantLinkRequest() # UpsertVariantLinkRequest | 

    try:
        # Upsert a variant procurement link
        api_response = api_instance.upsert_variant_link(upsert_variant_link_request)
        print("The response of ProcurementApi->upsert_variant_link:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling ProcurementApi->upsert_variant_link: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **upsert_variant_link_request** | [**UpsertVariantLinkRequest**](UpsertVariantLinkRequest.md)|  | 

### Return type

[**VariantLinkResponse**](VariantLinkResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | Link created/updated |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

