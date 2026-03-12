# ToolResultSubmission

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**is_error** | Option<**bool**> | Set to `true` if the tool execution failed. The error message in `result` is shown to the LLM so it can attempt recovery or report the failure. | [optional]
**result** | **String** | The tool execution result as a string. For structured data, serialize to JSON string. The content is passed directly to the LLM as the tool's output. | 
**tool_id** | **String** | The `tool_id` from the `tool_call_pending` SSE event that this result corresponds to. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


