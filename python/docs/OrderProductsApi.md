# tachyon_sdk.OrderProductsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_product**](OrderProductsApi.md#create_product) | **POST** /v1/order/products | Create a new product
[**delete_product**](OrderProductsApi.md#delete_product) | **DELETE** /v1/order/products/{id} | Delete a product by ID
[**get_product**](OrderProductsApi.md#get_product) | **GET** /v1/order/products/{id} | Get a product by ID
[**list_product_variants**](OrderProductsApi.md#list_product_variants) | **GET** /v1/order/products/{id}/variants | List product variants by product ID
[**list_products**](OrderProductsApi.md#list_products) | **GET** /v1/order/products | List all products with pagination
[**update_product**](OrderProductsApi.md#update_product) | **PUT** /v1/order/products/{id} | Update a product by ID


# **create_product**
> ProductResponse create_product(create_product_request)

Create a new product

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_product_request import CreateProductRequest
from tachyon_sdk.models.product_response import ProductResponse
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
    api_instance = tachyon_sdk.OrderProductsApi(api_client)
    create_product_request = tachyon_sdk.CreateProductRequest() # CreateProductRequest | 

    try:
        # Create a new product
        api_response = api_instance.create_product(create_product_request)
        print("The response of OrderProductsApi->create_product:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderProductsApi->create_product: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_product_request** | [**CreateProductRequest**](CreateProductRequest.md)|  | 

### Return type

[**ProductResponse**](ProductResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | Product created |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_product**
> DeleteProductResponse delete_product(id)

Delete a product by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.delete_product_response import DeleteProductResponse
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
    api_instance = tachyon_sdk.OrderProductsApi(api_client)
    id = 'id_example' # str | Product ID

    try:
        # Delete a product by ID
        api_response = api_instance.delete_product(id)
        print("The response of OrderProductsApi->delete_product:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderProductsApi->delete_product: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Product ID | 

### Return type

[**DeleteProductResponse**](DeleteProductResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Product deleted |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_product**
> ProductResponse get_product(id)

Get a product by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.product_response import ProductResponse
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
    api_instance = tachyon_sdk.OrderProductsApi(api_client)
    id = 'id_example' # str | Product ID

    try:
        # Get a product by ID
        api_response = api_instance.get_product(id)
        print("The response of OrderProductsApi->get_product:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderProductsApi->get_product: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Product ID | 

### Return type

[**ProductResponse**](ProductResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Product found |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_product_variants**
> ProductVariantListResponse list_product_variants(id)

List product variants by product ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.product_variant_list_response import ProductVariantListResponse
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
    api_instance = tachyon_sdk.OrderProductsApi(api_client)
    id = 'id_example' # str | Product ID

    try:
        # List product variants by product ID
        api_response = api_instance.list_product_variants(id)
        print("The response of OrderProductsApi->list_product_variants:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderProductsApi->list_product_variants: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Product ID | 

### Return type

[**ProductVariantListResponse**](ProductVariantListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Product variant list |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_products**
> ProductListResponse list_products(limit=limit, offset=offset)

List all products with pagination

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.product_list_response import ProductListResponse
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
    api_instance = tachyon_sdk.OrderProductsApi(api_client)
    limit = 56 # int | Max items to return (optional)
    offset = 56 # int | Items to skip (optional)

    try:
        # List all products with pagination
        api_response = api_instance.list_products(limit=limit, offset=offset)
        print("The response of OrderProductsApi->list_products:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderProductsApi->list_products: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **limit** | **int**| Max items to return | [optional] 
 **offset** | **int**| Items to skip | [optional] 

### Return type

[**ProductListResponse**](ProductListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Product list |  -  |
**403** | Forbidden |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_product**
> ProductResponse update_product(id, update_product_request)

Update a product by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.product_response import ProductResponse
from tachyon_sdk.models.update_product_request import UpdateProductRequest
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
    api_instance = tachyon_sdk.OrderProductsApi(api_client)
    id = 'id_example' # str | Product ID
    update_product_request = tachyon_sdk.UpdateProductRequest() # UpdateProductRequest | 

    try:
        # Update a product by ID
        api_response = api_instance.update_product(id, update_product_request)
        print("The response of OrderProductsApi->update_product:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderProductsApi->update_product: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Product ID | 
 **update_product_request** | [**UpdateProductRequest**](UpdateProductRequest.md)|  | 

### Return type

[**ProductResponse**](ProductResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Product updated |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

