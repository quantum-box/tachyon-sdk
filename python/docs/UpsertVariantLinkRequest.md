# UpsertVariantLinkRequest

Request body for upserting a variant procurement link

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**metadata** | **object** |  | [optional] 
**procurement_code** | **str** | Procurement code | 
**supplier_id** | **str** | Supplier ID | 
**variant_id** | **str** | Product variant ID | 

## Example

```python
from tachyon_sdk.models.upsert_variant_link_request import UpsertVariantLinkRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpsertVariantLinkRequest from a JSON string
upsert_variant_link_request_instance = UpsertVariantLinkRequest.from_json(json)
# print the JSON string representation of the object
print(UpsertVariantLinkRequest.to_json())

# convert the object into a dict
upsert_variant_link_request_dict = upsert_variant_link_request_instance.to_dict()
# create an instance of UpsertVariantLinkRequest from a dict
upsert_variant_link_request_from_dict = UpsertVariantLinkRequest.from_dict(upsert_variant_link_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


