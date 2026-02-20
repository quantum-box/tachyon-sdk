# tachyon_sdk.OrderShippingApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**register_shipping_destination**](OrderShippingApi.md#register_shipping_destination) | **POST** /v1/order/shipping-destinations | Register a shipping destination for a quote


# **register_shipping_destination**
> ShippingDestinationResponse register_shipping_destination(register_shipping_destination_request)

Register a shipping destination for a quote

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.register_shipping_destination_request import RegisterShippingDestinationRequest
from tachyon_sdk.models.shipping_destination_response import ShippingDestinationResponse
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
    api_instance = tachyon_sdk.OrderShippingApi(api_client)
    register_shipping_destination_request = tachyon_sdk.RegisterShippingDestinationRequest() # RegisterShippingDestinationRequest | 

    try:
        # Register a shipping destination for a quote
        api_response = api_instance.register_shipping_destination(register_shipping_destination_request)
        print("The response of OrderShippingApi->register_shipping_destination:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderShippingApi->register_shipping_destination: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **register_shipping_destination_request** | [**RegisterShippingDestinationRequest**](RegisterShippingDestinationRequest.md)|  | 

### Return type

[**ShippingDestinationResponse**](ShippingDestinationResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | Shipping destination registered |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

