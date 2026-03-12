# ConsentListResponse

Response for listing user consents.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**consents** | [**List[ConsentResponse]**](ConsentResponse.md) |  | 

## Example

```python
from tachyon_sdk.models.consent_list_response import ConsentListResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ConsentListResponse from a JSON string
consent_list_response_instance = ConsentListResponse.from_json(json)
# print the JSON string representation of the object
print(ConsentListResponse.to_json())

# convert the object into a dict
consent_list_response_dict = consent_list_response_instance.to_dict()
# create an instance of ConsentListResponse from a dict
consent_list_response_from_dict = ConsentListResponse.from_dict(consent_list_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


