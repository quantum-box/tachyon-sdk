# ChatroomsChatroomIdMessagesGetResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**messages** | [**List[ChatMessage]**](ChatMessage.md) |  | 

## Example

```python
from tachyon_sdk.models.chatrooms_chatroom_id_messages_get_response import ChatroomsChatroomIdMessagesGetResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ChatroomsChatroomIdMessagesGetResponse from a JSON string
chatrooms_chatroom_id_messages_get_response_instance = ChatroomsChatroomIdMessagesGetResponse.from_json(json)
# print the JSON string representation of the object
print(ChatroomsChatroomIdMessagesGetResponse.to_json())

# convert the object into a dict
chatrooms_chatroom_id_messages_get_response_dict = chatrooms_chatroom_id_messages_get_response_instance.to_dict()
# create an instance of ChatroomsChatroomIdMessagesGetResponse from a dict
chatrooms_chatroom_id_messages_get_response_from_dict = ChatroomsChatroomIdMessagesGetResponse.from_dict(chatrooms_chatroom_id_messages_get_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


