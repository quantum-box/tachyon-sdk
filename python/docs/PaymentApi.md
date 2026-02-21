# tachyon_sdk.PaymentApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_or_create_stripe_customer**](PaymentApi.md#get_or_create_stripe_customer) | **POST** /v1/payment/stripe-customer | Get or create a Stripe customer
[**list_providers**](PaymentApi.md#list_providers) | **GET** /v1/payment/providers | List payment providers by entity ID


# **get_or_create_stripe_customer**
> StripeCustomerResponse get_or_create_stripe_customer(stripe_customer_request)

Get or create a Stripe customer

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.stripe_customer_request import StripeCustomerRequest
from tachyon_sdk.models.stripe_customer_response import StripeCustomerResponse
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
    api_instance = tachyon_sdk.PaymentApi(api_client)
    stripe_customer_request = tachyon_sdk.StripeCustomerRequest() # StripeCustomerRequest | 

    try:
        # Get or create a Stripe customer
        api_response = api_instance.get_or_create_stripe_customer(stripe_customer_request)
        print("The response of PaymentApi->get_or_create_stripe_customer:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PaymentApi->get_or_create_stripe_customer: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **stripe_customer_request** | [**StripeCustomerRequest**](StripeCustomerRequest.md)|  | 

### Return type

[**StripeCustomerResponse**](StripeCustomerResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Stripe customer |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **list_providers**
> ProviderListResponse list_providers(entity_id)

List payment providers by entity ID

### Example


```python
import tachyon_sdk
from tachyon_sdk.models.provider_list_response import ProviderListResponse
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
    api_instance = tachyon_sdk.PaymentApi(api_client)
    entity_id = 'entity_id_example' # str | Entity ID

    try:
        # List payment providers by entity ID
        api_response = api_instance.list_providers(entity_id)
        print("The response of PaymentApi->list_providers:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PaymentApi->list_providers: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **entity_id** | **str**| Entity ID | 

### Return type

[**ProviderListResponse**](ProviderListResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Provider list |  -  |
**400** | Bad request |  -  |
**403** | Forbidden |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

