# tachyon_sdk.OrderPurchaseOrdersApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_purchase_order**](OrderPurchaseOrdersApi.md#get_purchase_order) | **GET** /v1/order/purchase-orders/{id} | Get a purchase order by ID
[**list_purchase_orders**](OrderPurchaseOrdersApi.md#list_purchase_orders) | **GET** /v1/order/purchase-orders | List all purchase orders


# **get_purchase_order**
> PurchaseOrderResponse get_purchase_order(id)

Get a purchase order by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.purchase_order_response import PurchaseOrderResponse
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
    api_instance = tachyon_sdk.OrderPurchaseOrdersApi(api_client)
    id = 'id_example' # str | Purchase order ID

    try:
        # Get a purchase order by ID
        api_response = api_instance.get_purchase_order(id)
        print("The response of OrderPurchaseOrdersApi->get_purchase_order:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderPurchaseOrdersApi->get_purchase_order: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Purchase order ID | 

### Return type

[**PurchaseOrderResponse**](PurchaseOrderResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Purchase order found |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_purchase_orders**
> PurchaseOrderListResponse list_purchase_orders()

List all purchase orders

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.purchase_order_list_response import PurchaseOrderListResponse
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
    api_instance = tachyon_sdk.OrderPurchaseOrdersApi(api_client)

    try:
        # List all purchase orders
        api_response = api_instance.list_purchase_orders()
        print("The response of OrderPurchaseOrdersApi->list_purchase_orders:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderPurchaseOrdersApi->list_purchase_orders: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**PurchaseOrderListResponse**](PurchaseOrderListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Purchase order list |  -  |
**403** | Forbidden |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

