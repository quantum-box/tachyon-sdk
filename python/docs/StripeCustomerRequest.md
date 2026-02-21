# StripeCustomerRequest

Request body for getting or creating a Stripe customer

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**address** | [**BillingAddressRequest**](BillingAddressRequest.md) | Billing address | [optional] 
**email** | **str** | Billing email | [optional] 
**name** | **str** | Billing name | [optional] 
**phone** | **str** | Phone number | [optional] 

## Example

```python
from tachyon_sdk.models.stripe_customer_request import StripeCustomerRequest

# TODO update the JSON string below
json = "{}"
# create an instance of StripeCustomerRequest from a JSON string
stripe_customer_request_instance = StripeCustomerRequest.from_json(json)
# print the JSON string representation of the object
print(StripeCustomerRequest.to_json())

# convert the object into a dict
stripe_customer_request_dict = stripe_customer_request_instance.to_dict()
# create an instance of StripeCustomerRequest from a dict
stripe_customer_request_from_dict = StripeCustomerRequest.from_dict(stripe_customer_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


