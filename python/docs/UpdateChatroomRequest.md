# UpdateChatroomRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**metadata** | [**OneOf**](OneOf.md) |  | [optional] 
**name** | **str** | Updated chatroom name | [optional] 

## Example

```python
from tachyon_sdk.models.update_chatroom_request import UpdateChatroomRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateChatroomRequest from a JSON string
update_chatroom_request_instance = UpdateChatroomRequest.from_json(json)
# print the JSON string representation of the object
print(UpdateChatroomRequest.to_json())

# convert the object into a dict
update_chatroom_request_dict = update_chatroom_request_instance.to_dict()
# create an instance of UpdateChatroomRequest from a dict
update_chatroom_request_from_dict = UpdateChatroomRequest.from_dict(update_chatroom_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


