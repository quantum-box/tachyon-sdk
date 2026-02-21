# StripeCustomerResponse

Stripe customer response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**default_payment_method_id** | **str** | Default payment method ID | [optional] 
**stripe_customer_id** | **str** | Stripe customer ID | 
**stripe_subscription_id** | **str** | Stripe subscription ID | [optional] 
**stripe_subscription_item_id** | **str** | Stripe subscription item ID | [optional] 

## Example

```python
from tachyon_sdk.models.stripe_customer_response import StripeCustomerResponse

# TODO update the JSON string below
json = "{}"
# create an instance of StripeCustomerResponse from a JSON string
stripe_customer_response_instance = StripeCustomerResponse.from_json(json)
# print the JSON string representation of the object
print(StripeCustomerResponse.to_json())

# convert the object into a dict
stripe_customer_response_dict = stripe_customer_response_instance.to_dict()
# create an instance of StripeCustomerResponse from a dict
stripe_customer_response_from_dict = StripeCustomerResponse.from_dict(stripe_customer_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


