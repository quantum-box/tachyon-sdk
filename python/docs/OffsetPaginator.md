# OffsetPaginator


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**current_page** | **int** |  | 
**items_per_page** | **int** |  | 
**total_items** | **int** |  | 
**total_pages** | **int** |  | 

## Example

```python
from tachyon_sdk.models.offset_paginator import OffsetPaginator

# TODO update the JSON string below
json = "{}"
# create an instance of OffsetPaginator from a JSON string
offset_paginator_instance = OffsetPaginator.from_json(json)
# print the JSON string representation of the object
print(OffsetPaginator.to_json())

# convert the object into a dict
offset_paginator_dict = offset_paginator_instance.to_dict()
# create an instance of OffsetPaginator from a dict
offset_paginator_from_dict = OffsetPaginator.from_dict(offset_paginator_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


