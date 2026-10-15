# BindingSlot

A server-derived slot.  `partyRef` is deliberately role-bound.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**party_ref** | [**AuthorityPartyRef**](AuthorityPartyRef.md) |  | [optional] 
**role** | [**BindingSlotRole**](BindingSlotRole.md) |  | 
**tenant_id** | **str** |  | 

## Example

```python
from tachyon_sdk.models.binding_slot import BindingSlot

# TODO update the JSON string below
json = "{}"
# create an instance of BindingSlot from a JSON string
binding_slot_instance = BindingSlot.from_json(json)
# print the JSON string representation of the object
print(BindingSlot.to_json())

# convert the object into a dict
binding_slot_dict = binding_slot_instance.to_dict()
# create an instance of BindingSlot from a dict
binding_slot_from_dict = BindingSlot.from_dict(binding_slot_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


