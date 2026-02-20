# CreateChatRoomResponse

Create chatroom response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chatroom** | [**ChatRoom**](ChatRoom.md) | Created chatroom | 

## Example

```python
from tachyon_sdk.models.create_chat_room_response import CreateChatRoomResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CreateChatRoomResponse from a JSON string
create_chat_room_response_instance = CreateChatRoomResponse.from_json(json)
# print the JSON string representation of the object
print(CreateChatRoomResponse.to_json())

# convert the object into a dict
create_chat_room_response_dict = create_chat_room_response_instance.to_dict()
# create an instance of CreateChatRoomResponse from a dict
create_chat_room_response_from_dict = CreateChatRoomResponse.from_dict(create_chat_room_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


