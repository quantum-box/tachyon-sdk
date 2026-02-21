# ProcurementApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**deleteVariantLink**](ProcurementApi.md#deletevariantlink) | **DELETE** /v1/procurement/variant-links/{id} | Delete a variant procurement link |
| [**upsertVariantLink**](ProcurementApi.md#upsertvariantlinkoperation) | **POST** /v1/procurement/variant-links | Upsert a variant procurement link |



## deleteVariantLink

> DeleteResponse deleteVariantLink(id)

Delete a variant procurement link

### Example

```ts
import {
  Configuration,
  ProcurementApi,
} from '@tachyon/sdk';
import type { DeleteVariantLinkRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new ProcurementApi();

  const body = {
    // string | Link ID
    id: id_example,
  } satisfies DeleteVariantLinkRequest;

  try {
    const data = await api.deleteVariantLink(body);
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
| **id** | `string` | Link ID | [Defaults to `undefined`] |

### Return type

[**DeleteResponse**](DeleteResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Link deleted |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## upsertVariantLink

> VariantLinkResponse upsertVariantLink(upsertVariantLinkRequest)

Upsert a variant procurement link

### Example

```ts
import {
  Configuration,
  ProcurementApi,
} from '@tachyon/sdk';
import type { UpsertVariantLinkOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new ProcurementApi();

  const body = {
    // UpsertVariantLinkRequest
    upsertVariantLinkRequest: ...,
  } satisfies UpsertVariantLinkOperationRequest;

  try {
    const data = await api.upsertVariantLink(body);
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
| **upsertVariantLinkRequest** | [UpsertVariantLinkRequest](UpsertVariantLinkRequest.md) |  | |

### Return type

[**VariantLinkResponse**](VariantLinkResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Link created/updated |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

