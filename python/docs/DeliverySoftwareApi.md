# tachyon_sdk.DeliverySoftwareApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_software_delivery_by_order**](DeliverySoftwareApi.md#get_software_delivery_by_order) | **GET** /v1/delivery/software/by-order/{order_id} | Get software delivery by order ID


# **get_software_delivery_by_order**
> SoftwareDeliveryResponse get_software_delivery_by_order(order_id)

Get software delivery by order ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.software_delivery_response import SoftwareDeliveryResponse
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
    api_instance = tachyon_sdk.DeliverySoftwareApi(api_client)
    order_id = 'order_id_example' # str | Purchase order ID

    try:
        # Get software delivery by order ID
        api_response = api_instance.get_software_delivery_by_order(order_id)
        print("The response of DeliverySoftwareApi->get_software_delivery_by_order:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DeliverySoftwareApi->get_software_delivery_by_order: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **order_id** | **str**| Purchase order ID | 

### Return type

[**SoftwareDeliveryResponse**](SoftwareDeliveryResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Software delivery found |  -  |
**404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

