# ToolResultSubmission

Result submitted by the client for a pending tool call.  After receiving a `tool_call_pending` SSE event, the client executes the tool and submits the result via `POST /v1/llms/chatrooms/{chatroom_id}/agent/tool-result`.  # Example  ```json {   \"tool_id\": \"ct_01JEXAMPLE\",   \"result\": \"{\\\"rows\\\": [{\\\"id\\\": 1, \\\"name\\\": \\\"Alice\\\"}]}\",   \"is_error\": false } ```

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**is_error** | **bool** | Set to &#x60;true&#x60; if the tool execution failed. The error message in &#x60;result&#x60; is shown to the LLM so it can attempt recovery or report the failure. | [optional] 
**result** | **str** | The tool execution result as a string. For structured data, serialize to JSON string. The content is passed directly to the LLM as the tool&#39;s output. | 
**tool_id** | **str** | The &#x60;tool_id&#x60; from the &#x60;tool_call_pending&#x60; SSE event that this result corresponds to. | 

## Example

```python
from tachyon_sdk.models.tool_result_submission import ToolResultSubmission

# TODO update the JSON string below
json = "{}"
# create an instance of ToolResultSubmission from a JSON string
tool_result_submission_instance = ToolResultSubmission.from_json(json)
# print the JSON string representation of the object
print(ToolResultSubmission.to_json())

# convert the object into a dict
tool_result_submission_dict = tool_result_submission_instance.to_dict()
# create an instance of ToolResultSubmission from a dict
tool_result_submission_from_dict = ToolResultSubmission.from_dict(tool_result_submission_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


