# tachyon_sdk.OrderQuotesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_quote**](OrderQuotesApi.md#create_quote) | **POST** /v1/order/quotes | Create a new quote
[**get_quote**](OrderQuotesApi.md#get_quote) | **GET** /v1/order/quotes/{id} | Get a quote by ID
[**issue_quote**](OrderQuotesApi.md#issue_quote) | **POST** /v1/order/quotes/{id}/issue | Issue a quote to a client
[**list_quotes**](OrderQuotesApi.md#list_quotes) | **GET** /v1/order/quotes | List all quotes
[**update_quote**](OrderQuotesApi.md#update_quote) | **PUT** /v1/order/quotes/{id} | Update a quote by ID


# **create_quote**
> QuoteResponse create_quote(create_quote_request)

Create a new quote

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.create_quote_request import CreateQuoteRequest
from tachyon_sdk.models.quote_response import QuoteResponse
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
    api_instance = tachyon_sdk.OrderQuotesApi(api_client)
    create_quote_request = tachyon_sdk.CreateQuoteRequest() # CreateQuoteRequest | 

    try:
        # Create a new quote
        api_response = api_instance.create_quote(create_quote_request)
        print("The response of OrderQuotesApi->create_quote:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderQuotesApi->create_quote: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_quote_request** | [**CreateQuoteRequest**](CreateQuoteRequest.md)|  | 

### Return type

[**QuoteResponse**](QuoteResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | Quote created |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_quote**
> QuoteResponse get_quote(id)

Get a quote by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.quote_response import QuoteResponse
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
    api_instance = tachyon_sdk.OrderQuotesApi(api_client)
    id = 'id_example' # str | Quote ID

    try:
        # Get a quote by ID
        api_response = api_instance.get_quote(id)
        print("The response of OrderQuotesApi->get_quote:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderQuotesApi->get_quote: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Quote ID | 

### Return type

[**QuoteResponse**](QuoteResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Quote found |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **issue_quote**
> QuoteResponse issue_quote(id, issue_quote_request)

Issue a quote to a client

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.issue_quote_request import IssueQuoteRequest
from tachyon_sdk.models.quote_response import QuoteResponse
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
    api_instance = tachyon_sdk.OrderQuotesApi(api_client)
    id = 'id_example' # str | Quote ID
    issue_quote_request = tachyon_sdk.IssueQuoteRequest() # IssueQuoteRequest | 

    try:
        # Issue a quote to a client
        api_response = api_instance.issue_quote(id, issue_quote_request)
        print("The response of OrderQuotesApi->issue_quote:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderQuotesApi->issue_quote: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Quote ID | 
 **issue_quote_request** | [**IssueQuoteRequest**](IssueQuoteRequest.md)|  | 

### Return type

[**QuoteResponse**](QuoteResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Quote issued |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_quotes**
> QuoteListResponse list_quotes()

List all quotes

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.quote_list_response import QuoteListResponse
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
    api_instance = tachyon_sdk.OrderQuotesApi(api_client)

    try:
        # List all quotes
        api_response = api_instance.list_quotes()
        print("The response of OrderQuotesApi->list_quotes:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderQuotesApi->list_quotes: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**QuoteListResponse**](QuoteListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Quote list |  -  |
**403** | Forbidden |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_quote**
> QuoteResponse update_quote(id, update_quote_request)

Update a quote by ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.quote_response import QuoteResponse
from tachyon_sdk.models.update_quote_request import UpdateQuoteRequest
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
    api_instance = tachyon_sdk.OrderQuotesApi(api_client)
    id = 'id_example' # str | Quote ID
    update_quote_request = tachyon_sdk.UpdateQuoteRequest() # UpdateQuoteRequest | 

    try:
        # Update a quote by ID
        api_response = api_instance.update_quote(id, update_quote_request)
        print("The response of OrderQuotesApi->update_quote:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OrderQuotesApi->update_quote: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **id** | **str**| Quote ID | 
 **update_quote_request** | [**UpdateQuoteRequest**](UpdateQuoteRequest.md)|  | 

### Return type

[**QuoteResponse**](QuoteResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Quote updated |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |
**404** | Not found |  -  |
**500** | Internal error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

