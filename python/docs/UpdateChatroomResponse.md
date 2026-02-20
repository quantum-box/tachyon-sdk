# UpdateChatroomResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chatroom** | [**ChatRoom**](ChatRoom.md) | Updated chatroom | 

## Example

```python
from tachyon_sdk.models.update_chatroom_response import UpdateChatroomResponse

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateChatroomResponse from a JSON string
update_chatroom_response_instance = UpdateChatroomResponse.from_json(json)
# print the JSON string representation of the object
print(UpdateChatroomResponse.to_json())

# convert the object into a dict
update_chatroom_response_dict = update_chatroom_response_instance.to_dict()
# create an instance of UpdateChatroomResponse from a dict
update_chatroom_response_from_dict = UpdateChatroomResponse.from_dict(update_chatroom_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


