# OAuth2CreateClientRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allowed_scopes** | **Vec<String>** | Allowed scopes |
**auth_mode** | Option<**String**> | Authentication mode: \"direct\" or \"proxied\" | [optional]
**grant_types** | **Vec<String>** | Allowed grant types |
**name** | **String** | Display name |
**redirect_uris** | **Vec<String>** | Allowed redirect URIs |
**use_tachyon_user_pool** | Option<**bool**> | Whether to use Tachyon user pool | [optional][default to true]
**user_pool_id** | Option<**String**> | Associated User Pool ID when using a dedicated pool | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


