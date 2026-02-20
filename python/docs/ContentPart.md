# ContentPart

Content part type for array messages

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**text** | **str** | The text content if type is \&quot;text\&quot; | [optional] 
**type** | **str** | The type of the content part | 

## Example

```python
from tachyon_sdk.models.content_part import ContentPart

# TODO update the JSON string below
json = "{}"
# create an instance of ContentPart from a JSON string
content_part_instance = ContentPart.from_json(json)
# print the JSON string representation of the object
print(ContentPart.to_json())

# convert the object into a dict
content_part_dict = content_part_instance.to_dict()
# create an instance of ContentPart from a dict
content_part_from_dict = ContentPart.from_dict(content_part_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


