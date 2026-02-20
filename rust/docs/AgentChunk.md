# AgentChunk

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tool_id** | **String** | Unique identifier for this tool call. Use this value as `tool_id` when submitting the result. | 
**tool_name** | **String** | Name of the client-defined tool the LLM wants to call. | 
**r#type** | **Type** |  (enum: usage) | 
**args** | Option<**serde_json::Value**> |  | 
**is_finished** | **bool** |  | 
**result** | **String** |  | 
**fire_and_forget** | Option<**bool**> | When `true`, the server does not wait for the client to submit a tool result — the LLM continues immediately. The client may still execute the tool for its side effects. | [optional]
**index** | **i32** |  | 
**text** | **String** |  | 
**created_at** | **String** |  | 
**id** | **String** |  | 
**user_id** | **String** |  | 
**options** | **Vec<String>** |  | 
**command** | Option<**String**> |  | [optional]
**cache_creation_input_tokens** | Option<**i32**> |  | [optional]
**cache_read_input_tokens** | Option<**i32**> |  | [optional]
**completion_tokens** | **i32** |  | 
**prompt_tokens** | **i32** |  | 
**total_cost** | Option<**f64**> |  | [optional]
**total_tokens** | **i32** |  | 
**agent** | Option<[**models::AgentSource**](AgentSource.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


