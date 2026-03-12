# ConnectionResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**connected_at** | **str** |  | 
**error_message** | **str** |  | [optional] 
**external_account_id** | **str** |  | [optional] 
**external_account_name** | **str** |  | [optional] 
**id** | **str** |  | 
**integration_id** | **str** |  | 
**last_synced_at** | **str** |  | [optional] 
**metadata** | **Dict[str, object]** |  | 
**provider** | **str** |  | 
**status** | **str** |  | 

## Example

```python
from tachyon_sdk.models.connection_response import ConnectionResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ConnectionResponse from a JSON string
connection_response_instance = ConnectionResponse.from_json(json)
# print the JSON string representation of the object
print(ConnectionResponse.to_json())

# convert the object into a dict
connection_response_dict = connection_response_instance.to_dict()
# create an instance of ConnectionResponse from a dict
connection_response_from_dict = ConnectionResponse.from_dict(connection_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


