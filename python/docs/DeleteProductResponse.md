# DeleteProductResponse

Delete product response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**success** | **bool** | Whether the deletion succeeded | 

## Example

```python
from tachyon_sdk.models.delete_product_response import DeleteProductResponse

# TODO update the JSON string below
json = "{}"
# create an instance of DeleteProductResponse from a JSON string
delete_product_response_instance = DeleteProductResponse.from_json(json)
# print the JSON string representation of the object
print(DeleteProductResponse.to_json())

# convert the object into a dict
delete_product_response_dict = delete_product_response_instance.to_dict()
# create an instance of DeleteProductResponse from a dict
delete_product_response_from_dict = DeleteProductResponse.from_dict(delete_product_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


