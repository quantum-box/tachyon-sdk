# Ask

Follow-up question with options

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**options** | **List[str]** |  | 
**text** | **str** |  | 

## Example

```python
from tachyon_sdk.models.ask import Ask

# TODO update the JSON string below
json = "{}"
# create an instance of Ask from a JSON string
ask_instance = Ask.from_json(json)
# print the JSON string representation of the object
print(Ask.to_json())

# convert the object into a dict
ask_dict = ask_instance.to_dict()
# create an instance of Ask from a dict
ask_from_dict = Ask.from_dict(ask_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


