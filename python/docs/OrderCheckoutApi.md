# tachyon_sdk.OrderCheckoutApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**process_checkout**](OrderCheckoutApi.md#process_checkout) | **POST** /v1/order/checkout | Process payment checkout for a quote


# **process_checkout**
> CheckoutResponse process_checkout(checkout_request)

Process payment checkout for a quote

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.checkout_request import CheckoutRequest
from tachyon_sdk.models.checkout_response import CheckoutResponse
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
    api_instance = tachyon_sdk.OrderCheckoutApi(api_client)
    checkout_request = tachyon_sdk.CheckoutRequest() # CheckoutRequest | 

    try:
        # Process payment checkout for a quote
        api_response = api_instance.process_checkout(checkout_request)
        print("The response of OrderCheckoutApi->process_checkout:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderCheckoutApi->process_checkout: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **checkout_request** | [**CheckoutRequest**](CheckoutRequest.md)|  | 

### Return type

[**CheckoutResponse**](CheckoutResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Checkout session created |  -  |
**400** | Bad request |  -  |
**402** | Payment required |  -  |
**403** | Forbidden |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

