# CRMObjectMappingsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createObjectMapping**](CRMObjectMappingsApi.md#createobjectmappingoperation) | **POST** /v1/crm/object-mappings | Create an object mapping |
| [**getObjectMappings**](CRMObjectMappingsApi.md#getobjectmappings) | **GET** /v1/crm/object-mappings | Get object mappings by entity ID and object name |



## createObjectMapping

> ObjectMappingResponse createObjectMapping(createObjectMappingRequest)

Create an object mapping

### Example

```ts
import {
  Configuration,
  CRMObjectMappingsApi,
} from '@tachyon/sdk';
import type { CreateObjectMappingOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new CRMObjectMappingsApi();

  const body = {
    // CreateObjectMappingRequest
    createObjectMappingRequest: ...,
  } satisfies CreateObjectMappingOperationRequest;

  try {
    const data = await api.createObjectMapping(body);
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
| **createObjectMappingRequest** | [CreateObjectMappingRequest](CreateObjectMappingRequest.md) |  | |

### Return type

[**ObjectMappingResponse**](ObjectMappingResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Object mapping created |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getObjectMappings

> ObjectMappingListResponse getObjectMappings(entityId, objectName)

Get object mappings by entity ID and object name

### Example

```ts
import {
  Configuration,
  CRMObjectMappingsApi,
} from '@tachyon/sdk';
import type { GetObjectMappingsRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new CRMObjectMappingsApi();

  const body = {
    // string | Entity ID to look up
    entityId: entityId_example,
    // string | Object type (Deal, Product, etc.)
    objectName: objectName_example,
  } satisfies GetObjectMappingsRequest;

  try {
    const data = await api.getObjectMappings(body);
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
| **entityId** | `string` | Entity ID to look up | [Defaults to `undefined`] |
| **objectName** | `string` | Object type (Deal, Product, etc.) | [Defaults to `undefined`] |

### Return type

[**ObjectMappingListResponse**](ObjectMappingListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Object mapping list |  -  |
| **400** | Bad request |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

