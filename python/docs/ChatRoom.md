# ChatRoom


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**id** | **str** |  | 
**metadata** | **object** | Arbitrary JSON metadata for the chatroom. Common fields include: - &#x60;user_id&#x60;: External user identifier (e.g., LINE user ID) for filtering - &#x60;comparison_group_id&#x60;: ID to group chatrooms for model comparison - &#x60;comparison_group_name&#x60;: Human-readable name for the comparison group | 
**name** | **str** |  | 
**operator_id** | **str** |  | 
**owner_id** | **str** |  | 
**updated_at** | **datetime** |  | 

## Example

```python
from tachyon_sdk.models.chat_room import ChatRoom

# TODO update the JSON string below
json = "{}"
# create an instance of ChatRoom from a JSON string
chat_room_instance = ChatRoom.from_json(json)
# print the JSON string representation of the object
print(ChatRoom.to_json())

# convert the object into a dict
chat_room_dict = chat_room_instance.to_dict()
# create an instance of ChatRoom from a dict
chat_room_from_dict = ChatRoom.from_dict(chat_room_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


