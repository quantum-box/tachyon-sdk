# CheckoutResponse

Checkout response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**session_id** | **str** | Stripe session ID | 

## Example

```python
from tachyon_sdk.models.checkout_response import CheckoutResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CheckoutResponse from a JSON string
checkout_response_instance = CheckoutResponse.from_json(json)
# print the JSON string representation of the object
print(CheckoutResponse.to_json())

# convert the object into a dict
checkout_response_dict = checkout_response_instance.to_dict()
# create an instance of CheckoutResponse from a dict
checkout_response_from_dict = CheckoutResponse.from_dict(checkout_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


