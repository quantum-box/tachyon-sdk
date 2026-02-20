# tachyon_sdk.DeliveryShippingApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_shipping_availability**](DeliveryShippingApi.md#check_shipping_availability) | **GET** /v1/delivery/shipping-destinations/{id}/availability | Check physical shipping availability
[**create_shipping_destination**](DeliveryShippingApi.md#create_shipping_destination) | **POST** /v1/delivery/shipping-destinations | Create a shipping destination


# **check_shipping_availability**
> ShippingAvailabilityResponse check_shipping_availability(id)

Check physical shipping availability

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.shipping_availability_response import ShippingAvailabilityResponse
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
    api_instance = tachyon_sdk.DeliveryShippingApi(api_client)
    id = 'id_example' # str | Shipping destination ID

    try:
        # Check physical shipping availability
        api_response = api_instance.check_shipping_availability(id)
        print("The response of DeliveryShippingApi->check_shipping_availability:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DeliveryShippingApi->check_shipping_availability: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Shipping destination ID | 

### Return type

[**ShippingAvailabilityResponse**](ShippingAvailabilityResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Availability check result |  -  |
**400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_shipping_destination**
> ShippingDestinationResponse create_shipping_destination(create_shipping_destination_request)

Create a shipping destination

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_shipping_destination_request import CreateShippingDestinationRequest
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
    api_instance = tachyon_sdk.DeliveryShippingApi(api_client)
    create_shipping_destination_request = tachyon_sdk.CreateShippingDestinationRequest() # CreateShippingDestinationRequest | 

    try:
        # Create a shipping destination
        api_response = api_instance.create_shipping_destination(create_shipping_destination_request)
        print("The response of DeliveryShippingApi->create_shipping_destination:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling DeliveryShippingApi->create_shipping_destination: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_shipping_destination_request** | [**CreateShippingDestinationRequest**](CreateShippingDestinationRequest.md)|  | 

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
**201** | Shipping destination created |  -  |
**400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

