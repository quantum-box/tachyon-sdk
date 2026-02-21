# OrderQuotesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createQuote**](OrderQuotesApi.md#createquoteoperation) | **POST** /v1/order/quotes | Create a new quote |
| [**getQuote**](OrderQuotesApi.md#getquote) | **GET** /v1/order/quotes/{id} | Get a quote by ID |
| [**issueQuote**](OrderQuotesApi.md#issuequoteoperation) | **POST** /v1/order/quotes/{id}/issue | Issue a quote to a client |
| [**listQuotes**](OrderQuotesApi.md#listquotes) | **GET** /v1/order/quotes | List all quotes |
| [**updateQuote**](OrderQuotesApi.md#updatequoteoperation) | **PUT** /v1/order/quotes/{id} | Update a quote by ID |



## createQuote

> QuoteResponse createQuote(createQuoteRequest)

Create a new quote

### Example

```ts
import {
  Configuration,
  OrderQuotesApi,
} from '@tachyon/sdk';
import type { CreateQuoteOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderQuotesApi();

  const body = {
    // CreateQuoteRequest
    createQuoteRequest: ...,
  } satisfies CreateQuoteOperationRequest;

  try {
    const data = await api.createQuote(body);
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
| **createQuoteRequest** | [CreateQuoteRequest](CreateQuoteRequest.md) |  | |

### Return type

[**QuoteResponse**](QuoteResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Quote created |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getQuote

> QuoteResponse getQuote(id)

Get a quote by ID

### Example

```ts
import {
  Configuration,
  OrderQuotesApi,
} from '@tachyon/sdk';
import type { GetQuoteRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderQuotesApi();

  const body = {
    // string | Quote ID
    id: id_example,
  } satisfies GetQuoteRequest;

  try {
    const data = await api.getQuote(body);
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
| **id** | `string` | Quote ID | [Defaults to `undefined`] |

### Return type

[**QuoteResponse**](QuoteResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Quote found |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## issueQuote

> QuoteResponse issueQuote(id, issueQuoteRequest)

Issue a quote to a client

### Example

```ts
import {
  Configuration,
  OrderQuotesApi,
} from '@tachyon/sdk';
import type { IssueQuoteOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderQuotesApi();

  const body = {
    // string | Quote ID
    id: id_example,
    // IssueQuoteRequest
    issueQuoteRequest: ...,
  } satisfies IssueQuoteOperationRequest;

  try {
    const data = await api.issueQuote(body);
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
| **id** | `string` | Quote ID | [Defaults to `undefined`] |
| **issueQuoteRequest** | [IssueQuoteRequest](IssueQuoteRequest.md) |  | |

### Return type

[**QuoteResponse**](QuoteResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Quote issued |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listQuotes

> QuoteListResponse listQuotes()

List all quotes

### Example

```ts
import {
  Configuration,
  OrderQuotesApi,
} from '@tachyon/sdk';
import type { ListQuotesRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderQuotesApi();

  try {
    const data = await api.listQuotes();
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**QuoteListResponse**](QuoteListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Quote list |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateQuote

> QuoteResponse updateQuote(id, updateQuoteRequest)

Update a quote by ID

### Example

```ts
import {
  Configuration,
  OrderQuotesApi,
} from '@tachyon/sdk';
import type { UpdateQuoteOperationRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new OrderQuotesApi();

  const body = {
    // string | Quote ID
    id: id_example,
    // UpdateQuoteRequest
    updateQuoteRequest: ...,
  } satisfies UpdateQuoteOperationRequest;

  try {
    const data = await api.updateQuote(body);
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
| **id** | `string` | Quote ID | [Defaults to `undefined`] |
| **updateQuoteRequest** | [UpdateQuoteRequest](UpdateQuoteRequest.md) |  | |

### Return type

[**QuoteResponse**](QuoteResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Quote updated |  -  |
| **400** | Bad request |  -  |
| **403** | Forbidden |  -  |
| **404** | Not found |  -  |
| **500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

