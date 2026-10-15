# AuthorityActor

Actor identity and its source-owned representation.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**actor_id** | **str** |  | 
**representation** | **str** |  | 

## Example

```python
from tachyon_sdk.models.authority_actor import AuthorityActor

# TODO update the JSON string below
json = "{}"
# create an instance of AuthorityActor from a JSON string
authority_actor_instance = AuthorityActor.from_json(json)
# print the JSON string representation of the object
print(AuthorityActor.to_json())

# convert the object into a dict
authority_actor_dict = authority_actor_instance.to_dict()
# create an instance of AuthorityActor from a dict
authority_actor_from_dict = AuthorityActor.from_dict(authority_actor_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


