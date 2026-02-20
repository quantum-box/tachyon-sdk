# GetChatroomsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chatrooms** | [**List[ChatRoom]**](ChatRoom.md) |  | 
**paginator** | [**OffsetPaginator**](OffsetPaginator.md) |  | 

## Example

```python
from tachyon_sdk.models.get_chatrooms_response import GetChatroomsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetChatroomsResponse from a JSON string
get_chatrooms_response_instance = GetChatroomsResponse.from_json(json)
# print the JSON string representation of the object
print(GetChatroomsResponse.to_json())

# convert the object into a dict
get_chatrooms_response_dict = get_chatrooms_response_instance.to_dict()
# create an instance of GetChatroomsResponse from a dict
get_chatrooms_response_from_dict = GetChatroomsResponse.from_dict(get_chatrooms_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


