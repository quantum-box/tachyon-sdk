# AgentMessagesResponse

TODO: add English documentation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**messages** | [**List[AgentChunk]**](AgentChunk.md) | TODO: add English documentation | 

## Example

```python
from tachyon_sdk.models.agent_messages_response import AgentMessagesResponse

# TODO update the JSON string below
json = "{}"
# create an instance of AgentMessagesResponse from a JSON string
agent_messages_response_instance = AgentMessagesResponse.from_json(json)
# print the JSON string representation of the object
print(AgentMessagesResponse.to_json())

# convert the object into a dict
agent_messages_response_dict = agent_messages_response_instance.to_dict()
# create an instance of AgentMessagesResponse from a dict
agent_messages_response_from_dict = AgentMessagesResponse.from_dict(agent_messages_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


