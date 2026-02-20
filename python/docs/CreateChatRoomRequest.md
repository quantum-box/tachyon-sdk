# CreateChatRoomRequest

Create chatroom request

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**metadata** | [**OneOf**](OneOf.md) |  | [optional] 
**name** | **str** | Name of the chatroom | [optional] 

## Example

```python
from tachyon_sdk.models.create_chat_room_request import CreateChatRoomRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateChatRoomRequest from a JSON string
create_chat_room_request_instance = CreateChatRoomRequest.from_json(json)
# print the JSON string representation of the object
print(CreateChatRoomRequest.to_json())

# convert the object into a dict
create_chat_room_request_dict = create_chat_room_request_instance.to_dict()
# create an instance of CreateChatRoomRequest from a dict
create_chat_room_request_from_dict = CreateChatRoomRequest.from_dict(create_chat_room_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


