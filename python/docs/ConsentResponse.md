# ConsentResponse

Response for a single consent entry.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**client_id** | **str** |  | 
**client_name** | **str** |  | 
**consent_id** | **str** |  | 
**consented_at** | **str** |  | 
**granted_scopes** | **List[str]** |  | 
**revoked_at** | **str** |  | [optional] 

## Example

```python
from tachyon_sdk.models.consent_response import ConsentResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ConsentResponse from a JSON string
consent_response_instance = ConsentResponse.from_json(json)
# print the JSON string representation of the object
print(ConsentResponse.to_json())

# convert the object into a dict
consent_response_dict = consent_response_instance.to_dict()
# create an instance of ConsentResponse from a dict
consent_response_from_dict = ConsentResponse.from_dict(consent_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


