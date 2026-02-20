# Usage

Token usage statistics

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cache_creation_input_tokens** | **int** |  | [optional] 
**cache_read_input_tokens** | **int** |  | [optional] 
**completion_tokens** | **int** |  | 
**prompt_tokens** | **int** |  | 
**total_cost** | **float** |  | [optional] 
**total_tokens** | **int** |  | 

## Example

```python
from tachyon_sdk.models.usage import Usage

# TODO update the JSON string below
json = "{}"
# create an instance of Usage from a JSON string
usage_instance = Usage.from_json(json)
# print the JSON string representation of the object
print(Usage.to_json())

# convert the object into a dict
usage_dict = usage_instance.to_dict()
# create an instance of Usage from a dict
usage_from_dict = Usage.from_dict(usage_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


