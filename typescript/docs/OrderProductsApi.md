# OrderProductsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createProduct**](OrderProductsApi.md#createproductoperation) | **POST** /v1/order/products | Create a new product |
| [**deleteProduct**](OrderProductsApi.md#deleteproduct) | **DELETE** /v1/order/products/{id} | Delete a product by ID |
| [**getProduct**](OrderProductsApi.md#getproduct) | **GET** /v1/order/products/{id} | Get a product by ID |
| [**listProducts**](OrderProductsApi.md#listproducts) | **GET** /v1/order/products | List all products with pagination |
| [**updateProduct**](OrderProductsApi.md#updateproductoperation) | **PUT** /v1/order/products/{id} | Update a product by ID |



## createProduct

> ProductResponse createProduct(createProductRequest)

Create a new product

### Example

```ts
import {
  Configuration,
  OrderProductsApi,
} from '@tachyon/sdk';
import type { CreateProductOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderProductsApi();

  const body = {
    // CreateProductRequest
    createProductRequest: ...,
  } satisfies CreateProductOperationRequest;

  try {
    const data = await api.createProduct(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **createProductRequest** | [CreateProductRequest](CreateProductRequest.md) |  | |

### Return type

[**ProductResponse**](ProductResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Product created |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deleteProduct

> DeleteProductResponse deleteProduct(id)

Delete a product by ID

### Example

```ts
import {
  Configuration,
  OrderProductsApi,
} from '@tachyon/sdk';
import type { DeleteProductRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderProductsApi();

  const body = {
    // string | Product ID
    id: id_example,
  } satisfies DeleteProductRequest;

  try {
    const data = await api.deleteProduct(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | `string` | Product ID | [Defaults to `undefined`] |

### Return type

[**DeleteProductResponse**](DeleteProductResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Product deleted |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getProduct

> ProductResponse getProduct(id)

Get a product by ID

### Example

```ts
import {
  Configuration,
  OrderProductsApi,
} from '@tachyon/sdk';
import type { GetProductRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderProductsApi();

  const body = {
    // string | Product ID
    id: id_example,
  } satisfies GetProductRequest;

  try {
    const data = await api.getProduct(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | `string` | Product ID | [Defaults to `undefined`] |

### Return type

[**ProductResponse**](ProductResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Product found |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listProducts

> ProductListResponse listProducts(limit, offset)

List all products with pagination

### Example

```ts
import {
  Configuration,
  OrderProductsApi,
} from '@tachyon/sdk';
import type { ListProductsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderProductsApi();

  const body = {
    // number | Max items to return (optional)
    limit: 56,
    // number | Items to skip (optional)
    offset: 56,
  } satisfies ListProductsRequest;

  try {
    const data = await api.listProducts(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **limit** | `number` | Max items to return | [Optional] [Defaults to `undefined`] |
| **offset** | `number` | Items to skip | [Optional] [Defaults to `undefined`] |

### Return type

[**ProductListResponse**](ProductListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Product list |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateProduct

> ProductResponse updateProduct(id, updateProductRequest)

Update a product by ID

### Example

```ts
import {
  Configuration,
  OrderProductsApi,
} from '@tachyon/sdk';
import type { UpdateProductOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderProductsApi();

  const body = {
    // string | Product ID
    id: id_example,
    // UpdateProductRequest
    updateProductRequest: ...,
  } satisfies UpdateProductOperationRequest;

  try {
    const data = await api.updateProduct(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | `string` | Product ID | [Defaults to `undefined`] |
| **updateProductRequest** | [UpdateProductRequest](UpdateProductRequest.md) |  | |

### Return type

[**ProductResponse**](ProductResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Product updated |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

