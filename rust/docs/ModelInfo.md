# ModelInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**context_window** | Option<**i32**> | Maximum context window size in tokens. | [optional]
**description** | Option<**String**> | Human-readable description of the model. | [optional]
**id** | **String** | Unique model identifier in `provider/model-name` format. | 
**max_output_tokens** | Option<**i32**> | Maximum number of output tokens. | [optional]
**name** | **String** | Model name. | 
**provider** | **String** | Provider name (e.g. anthropic, openai, google_ai). | 
**supported_features** | [**Vec<models::SupportedFeature>**](SupportedFeature.md) | List of features supported by this model. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


