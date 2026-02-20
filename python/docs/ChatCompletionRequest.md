# ChatCompletionRequest

Request type for chat completion API

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**communication_style** | **str** | Communication style preference | [optional] 
**explanation_style** | **str** | Explanation style preference | [optional] 
**frequency_penalty** | **float** | Number between -2.0 and 2.0 | [optional] [default to 0]
**max_completion_tokens** | **int** | The maximum number of tokens to generate | [optional] [default to 10000]
**memory_settings** | [**MemorySettingsRequest**](MemorySettingsRequest.md) | Memory settings for context building | [optional] 
**messages** | [**List[Message]**](Message.md) | A list of messages comprising the conversation so far | 
**model** | **str** | ID of the model to use | [optional] 
**n** | **int** | How many chat completion choices to generate | [optional] [default to 1]
**presence_penalty** | **float** | Number between -2.0 and 2.0 | [optional] [default to 0]
**response_format** | [**ResponseFormat**](ResponseFormat.md) | Format to return the response in | [optional] 
**stream** | **bool** | Whether to stream back partial progress | [optional] [default to False]
**technical_level** | **str** | Technical level preference | [optional] 
**temperature** | **float** | What sampling temperature to use, between 0 and 2 | [optional] [default to 1.0]
**tool_choice** | [**ToolChoice**](ToolChoice.md) | Controls which (if any) function is called by the model | [optional] 
**tools** | [**List[Tool]**](Tool.md) | A list of tools the model may call | [optional] 
**top_p** | **float** | An alternative to sampling with temperature | [optional] [default to 1.0]
**user** | **str** | A unique identifier representing your end-user | [optional] 

## Example

```python
from tachyon_sdk.models.chat_completion_request import ChatCompletionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ChatCompletionRequest from a JSON string
chat_completion_request_instance = ChatCompletionRequest.from_json(json)
# print the JSON string representation of the object
print(ChatCompletionRequest.to_json())

# convert the object into a dict
chat_completion_request_dict = chat_completion_request_instance.to_dict()
# create an instance of ChatCompletionRequest from a dict
chat_completion_request_from_dict = ChatCompletionRequest.from_dict(chat_completion_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


