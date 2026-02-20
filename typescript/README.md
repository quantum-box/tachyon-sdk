# @tachyon/sdk@0.1.0

A TypeScript SDK client for the localhost API.

## Usage

First, install the SDK from npm.

```bash
npm install @tachyon/sdk --save
```

Next, try it out.


```ts
import {
  Configuration,
  AgentApi,
} from '@tachyon/sdk';
import type { ExecuteAgentRequest } from '@tachyon/sdk';

async function example() {
  console.log("🚀 Testing @tachyon/sdk SDK...");
  const api = new AgentApi();

  const body = {
    // string | Chatroom ID
    chatroomId: chatroomId_example,
    // string | Operator ID
    xOperatorId: tn_xxxxxx,
    // string | Authorization
    authorization: Bearer xxxxx,
    // AgentExecuteRequest
    agentExecuteRequest: ...,
  } satisfies ExecuteAgentRequest;

  try {
    const data = await api.executeAgent(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```


## Documentation

### API Endpoints

All URIs are relative to *http://localhost*

| Class | Method | HTTP request | Description
| ----- | ------ | ------------ | -------------
*AgentApi* | [**executeAgent**](docs/AgentApi.md#executeagent) | **POST** /v1/llms/chatrooms/{chatroom_id}/agent/execute | Execute agent
*AgentApi* | [**getAgentMessages**](docs/AgentApi.md#getagentmessages) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/messages | Get agent message log
*AgentApi* | [**getAgentStatus**](docs/AgentApi.md#getagentstatus) | **GET** /v1/llms/chatrooms/{chatroom_id}/agent/status | Get agent status
*AuthAPIKeysApi* | [**createApiKey**](docs/AuthAPIKeysApi.md#createapikeyoperation) | **POST** /v1/auth/service-accounts/{service_account_id}/api-keys | Create a new API key for a service account
*AuthAPIKeysApi* | [**listApiKeys**](docs/AuthAPIKeysApi.md#listapikeys) | **GET** /v1/auth/service-accounts/{service_account_id}/api-keys | List API keys for a service account
*AuthPoliciesApi* | [**evaluatePoliciesBatch**](docs/AuthPoliciesApi.md#evaluatepoliciesbatchoperation) | **POST** /v1/auth/policies/check | Evaluate multiple policy actions in batch
*AuthPoliciesApi* | [**getPolicy**](docs/AuthPoliciesApi.md#getpolicy) | **GET** /v1/auth/policies/{id} | Get a policy by ID
*AuthPoliciesApi* | [**listActions**](docs/AuthPoliciesApi.md#listactions) | **GET** /v1/auth/actions | List all registered actions
*AuthServiceAccountsApi* | [**createServiceAccount**](docs/AuthServiceAccountsApi.md#createserviceaccountoperation) | **POST** /v1/auth/service-accounts | Create a new service account
*AuthServiceAccountsApi* | [**deleteServiceAccount**](docs/AuthServiceAccountsApi.md#deleteserviceaccount) | **DELETE** /v1/auth/service-accounts/{id} | Delete a service account
*AuthServiceAccountsApi* | [**getServiceAccount**](docs/AuthServiceAccountsApi.md#getserviceaccount) | **GET** /v1/auth/service-accounts/{id} | Get a service account by ID
*AuthServiceAccountsApi* | [**listServiceAccounts**](docs/AuthServiceAccountsApi.md#listserviceaccounts) | **GET** /v1/auth/service-accounts | List all service accounts
*AuthServiceAccountsApi* | [**updateServiceAccount**](docs/AuthServiceAccountsApi.md#updateserviceaccountoperation) | **PUT** /v1/auth/service-accounts/{id} | Update a service account
*AuthUsersApi* | [**createUser**](docs/AuthUsersApi.md#createuseroperation) | **POST** /auth/v1beta/users | 
*AuthUsersApi* | [**getUser**](docs/AuthUsersApi.md#getuser) | **GET** /v1/auth/users/{id} | Get a user by ID
*AuthUsersApi* | [**listUsers**](docs/AuthUsersApi.md#listusers) | **GET** /v1/auth/users | List all users in an operator
*AuthVerifyApi* | [**signInWithPlatform**](docs/AuthVerifyApi.md#signinwithplatformoperation) | **POST** /auth/v1beta/sign-in-with-platform | 
*AuthVerifyApi* | [**verify**](docs/AuthVerifyApi.md#verifyoperation) | **POST** /auth/v1beta/verify | 
*CRMObjectMappingsApi* | [**createObjectMapping**](docs/CRMObjectMappingsApi.md#createobjectmappingoperation) | **POST** /v1/crm/object-mappings | Create an object mapping
*CRMObjectMappingsApi* | [**getObjectMappings**](docs/CRMObjectMappingsApi.md#getobjectmappings) | **GET** /v1/crm/object-mappings | Get object mappings by entity ID and object name
*ChatApi* | [**chatCompletion**](docs/ChatApi.md#chatcompletionoperation) | **POST** /v1/llms/chat/completions | Create a chat completion
*ChatApi* | [**chatCompletionOnChatroom**](docs/ChatApi.md#chatcompletiononchatroom) | **POST** /v1/llms/chatrooms/{chatroom_id}/chat/completions | Create a chat completion within a chatroom
*ChatroomApi* | [**createChatroom**](docs/ChatroomApi.md#createchatroom) | **POST** /v1/llms/chatrooms | Create a new chatroom
*ChatroomApi* | [**deleteChatroom**](docs/ChatroomApi.md#deletechatroom) | **DELETE** /v1/llms/chatrooms/{chatroom_id} | Delete a chatroom
*ChatroomApi* | [**updateChatroom**](docs/ChatroomApi.md#updatechatroomoperation) | **PATCH** /v1/llms/chatrooms/{chatroom_id} | Update a chatroom
*DefaultApi* | [**getChatroomMessages**](docs/DefaultApi.md#getchatroommessages) | **GET** /v1/llms/chatrooms/{chatroom_id}/messages | Get messages from a chatroom
*DefaultApi* | [**getChatrooms**](docs/DefaultApi.md#getchatrooms) | **GET** /v1/llms/chatrooms | Get chatrooms list
*DeliveryShippingApi* | [**checkShippingAvailability**](docs/DeliveryShippingApi.md#checkshippingavailability) | **GET** /v1/delivery/shipping-destinations/{id}/availability | Check physical shipping availability
*DeliveryShippingApi* | [**createShippingDestination**](docs/DeliveryShippingApi.md#createshippingdestinationoperation) | **POST** /v1/delivery/shipping-destinations | Create a shipping destination
*DeliverySoftwareApi* | [**getSoftwareDeliveryByOrder**](docs/DeliverySoftwareApi.md#getsoftwaredeliverybyorder) | **GET** /v1/delivery/software/by-order/{order_id} | Get software delivery by order ID
*FeatureFlagsApi* | [**evaluateActions**](docs/FeatureFlagsApi.md#evaluateactionsoperation) | **POST** /v1/feature-flags/actions/evaluate | TODO: add English documentation
*ModelsApi* | [**getModels**](docs/ModelsApi.md#getmodels) | **GET** /v1/llms/models | Get list of supported models
*OauthApi* | [**callback**](docs/OauthApi.md#callback) | **GET** /oauth/{provider_name}/callback | OAuth callback handler for specified provider
*OauthApi* | [**connect**](docs/OauthApi.md#connect) | **GET** /oauth/{provider_name}/connect | Get OAuth authorization URL for specified provider
*OrderCheckoutApi* | [**processCheckout**](docs/OrderCheckoutApi.md#processcheckout) | **POST** /v1/order/checkout | Process payment checkout for a quote
*OrderClientsApi* | [**createClient**](docs/OrderClientsApi.md#createclientoperation) | **POST** /v1/order/clients | Create a new client
*OrderClientsApi* | [**getClient**](docs/OrderClientsApi.md#getclient) | **GET** /v1/order/clients/{id} | Get a client by ID
*OrderClientsApi* | [**listClients**](docs/OrderClientsApi.md#listclients) | **GET** /v1/order/clients | List all clients
*OrderProductsApi* | [**createProduct**](docs/OrderProductsApi.md#createproductoperation) | **POST** /v1/order/products | Create a new product
*OrderProductsApi* | [**deleteProduct**](docs/OrderProductsApi.md#deleteproduct) | **DELETE** /v1/order/products/{id} | Delete a product by ID
*OrderProductsApi* | [**getProduct**](docs/OrderProductsApi.md#getproduct) | **GET** /v1/order/products/{id} | Get a product by ID
*OrderProductsApi* | [**listProducts**](docs/OrderProductsApi.md#listproducts) | **GET** /v1/order/products | List all products with pagination
*OrderProductsApi* | [**updateProduct**](docs/OrderProductsApi.md#updateproductoperation) | **PUT** /v1/order/products/{id} | Update a product by ID
*OrderPurchaseOrdersApi* | [**getPurchaseOrder**](docs/OrderPurchaseOrdersApi.md#getpurchaseorder) | **GET** /v1/order/purchase-orders/{id} | Get a purchase order by ID
*OrderPurchaseOrdersApi* | [**listPurchaseOrders**](docs/OrderPurchaseOrdersApi.md#listpurchaseorders) | **GET** /v1/order/purchase-orders | List all purchase orders
*OrderQuotesApi* | [**createQuote**](docs/OrderQuotesApi.md#createquoteoperation) | **POST** /v1/order/quotes | Create a new quote
*OrderQuotesApi* | [**getQuote**](docs/OrderQuotesApi.md#getquote) | **GET** /v1/order/quotes/{id} | Get a quote by ID
*OrderQuotesApi* | [**issueQuote**](docs/OrderQuotesApi.md#issuequoteoperation) | **POST** /v1/order/quotes/{id}/issue | Issue a quote to a client
*OrderQuotesApi* | [**listQuotes**](docs/OrderQuotesApi.md#listquotes) | **GET** /v1/order/quotes | List all quotes
*OrderShippingApi* | [**registerShippingDestination**](docs/OrderShippingApi.md#registershippingdestinationoperation) | **POST** /v1/order/shipping-destinations | Register a shipping destination for a quote


### Models

- [ActionEvaluationRequest](docs/ActionEvaluationRequest.md)
- [ActionEvaluationResult](docs/ActionEvaluationResult.md)
- [ActionListResponse](docs/ActionListResponse.md)
- [ActionResponse](docs/ActionResponse.md)
- [AddressRequest](docs/AddressRequest.md)
- [AddressResponse](docs/AddressResponse.md)
- [AgentChunk](docs/AgentChunk.md)
- [AgentChunkEvent](docs/AgentChunkEvent.md)
- [AgentChunkEventOneOf](docs/AgentChunkEventOneOf.md)
- [AgentChunkEventOneOf1](docs/AgentChunkEventOneOf1.md)
- [AgentChunkEventOneOf2](docs/AgentChunkEventOneOf2.md)
- [AgentChunkEventOneOf3](docs/AgentChunkEventOneOf3.md)
- [AgentChunkEventOneOf4](docs/AgentChunkEventOneOf4.md)
- [AgentChunkEventOneOf5](docs/AgentChunkEventOneOf5.md)
- [AgentChunkEventOneOf6](docs/AgentChunkEventOneOf6.md)
- [AgentChunkEventOneOf7](docs/AgentChunkEventOneOf7.md)
- [AgentChunkEventOneOf8](docs/AgentChunkEventOneOf8.md)
- [AgentChunkEventOneOf9](docs/AgentChunkEventOneOf9.md)
- [AgentExecuteRequest](docs/AgentExecuteRequest.md)
- [AgentMessage](docs/AgentMessage.md)
- [AgentMessagesResponse](docs/AgentMessagesResponse.md)
- [AgentProtocolMode](docs/AgentProtocolMode.md)
- [AgentSource](docs/AgentSource.md)
- [AgentStatusResponse](docs/AgentStatusResponse.md)
- [AgentToolAccessRequest](docs/AgentToolAccessRequest.md)
- [ApiKeyListResponse](docs/ApiKeyListResponse.md)
- [ApiKeyResponse](docs/ApiKeyResponse.md)
- [Ask](docs/Ask.md)
- [AttemptCompletionResult](docs/AttemptCompletionResult.md)
- [AuthUrlResponse](docs/AuthUrlResponse.md)
- [ChatCompletionChunkResponse](docs/ChatCompletionChunkResponse.md)
- [ChatCompletionRequest](docs/ChatCompletionRequest.md)
- [ChatCompletionResponse](docs/ChatCompletionResponse.md)
- [ChatCompletionWithChatroomStreamResponse](docs/ChatCompletionWithChatroomStreamResponse.md)
- [ChatMessage](docs/ChatMessage.md)
- [ChatRole](docs/ChatRole.md)
- [ChatRoom](docs/ChatRoom.md)
- [ChatroomNameGeneration](docs/ChatroomNameGeneration.md)
- [ChatroomsChatroomIdMessagesGetResponse](docs/ChatroomsChatroomIdMessagesGetResponse.md)
- [CheckoutRequest](docs/CheckoutRequest.md)
- [CheckoutResponse](docs/CheckoutResponse.md)
- [Choice](docs/Choice.md)
- [ChunkChoice](docs/ChunkChoice.md)
- [ClientListResponse](docs/ClientListResponse.md)
- [ClientResponse](docs/ClientResponse.md)
- [ClientToolDefinition](docs/ClientToolDefinition.md)
- [ContentPart](docs/ContentPart.md)
- [CreateApiKeyRequest](docs/CreateApiKeyRequest.md)
- [CreateChatRoomRequest](docs/CreateChatRoomRequest.md)
- [CreateChatRoomResponse](docs/CreateChatRoomResponse.md)
- [CreateClientRequest](docs/CreateClientRequest.md)
- [CreateObjectMappingRequest](docs/CreateObjectMappingRequest.md)
- [CreateProductRequest](docs/CreateProductRequest.md)
- [CreateQuoteRequest](docs/CreateQuoteRequest.md)
- [CreateServiceAccountRequest](docs/CreateServiceAccountRequest.md)
- [CreateShippingDestinationRequest](docs/CreateShippingDestinationRequest.md)
- [CreateUserRequest](docs/CreateUserRequest.md)
- [CreateUserResponse](docs/CreateUserResponse.md)
- [DeleteProductResponse](docs/DeleteProductResponse.md)
- [DeleteServiceAccountResponse](docs/DeleteServiceAccountResponse.md)
- [DeltaMessage](docs/DeltaMessage.md)
- [ErrorResponse](docs/ErrorResponse.md)
- [EvaluateActionsRequest](docs/EvaluateActionsRequest.md)
- [EvaluateActionsResponse](docs/EvaluateActionsResponse.md)
- [EvaluatePoliciesBatchRequest](docs/EvaluatePoliciesBatchRequest.md)
- [EvaluatePoliciesBatchResponse](docs/EvaluatePoliciesBatchResponse.md)
- [Function](docs/Function.md)
- [FunctionCall](docs/FunctionCall.md)
- [FunctionCallResponse](docs/FunctionCallResponse.md)
- [GetChatroomsResponse](docs/GetChatroomsResponse.md)
- [IssueQuoteRequest](docs/IssueQuoteRequest.md)
- [LineItemRequest](docs/LineItemRequest.md)
- [LineItemResponse](docs/LineItemResponse.md)
- [MemorySettingsRequest](docs/MemorySettingsRequest.md)
- [Message](docs/Message.md)
- [MessageContent](docs/MessageContent.md)
- [ModelInfo](docs/ModelInfo.md)
- [ModelsResponse](docs/ModelsResponse.md)
- [OAuthCallbackResponse](docs/OAuthCallbackResponse.md)
- [ObjectMappingItemResponse](docs/ObjectMappingItemResponse.md)
- [ObjectMappingListResponse](docs/ObjectMappingListResponse.md)
- [ObjectMappingResponse](docs/ObjectMappingResponse.md)
- [OffsetPaginator](docs/OffsetPaginator.md)
- [Part](docs/Part.md)
- [PartOneOf](docs/PartOneOf.md)
- [PartOneOf1](docs/PartOneOf1.md)
- [PartOneOf2](docs/PartOneOf2.md)
- [PartOneOf3](docs/PartOneOf3.md)
- [PartOneOf4](docs/PartOneOf4.md)
- [PolicyEvaluationOutcome](docs/PolicyEvaluationOutcome.md)
- [PolicyResponse](docs/PolicyResponse.md)
- [ProductListResponse](docs/ProductListResponse.md)
- [ProductResponse](docs/ProductResponse.md)
- [ProductVariationRequest](docs/ProductVariationRequest.md)
- [PurchaseOrderListResponse](docs/PurchaseOrderListResponse.md)
- [PurchaseOrderResponse](docs/PurchaseOrderResponse.md)
- [QuoteListResponse](docs/QuoteListResponse.md)
- [QuoteResponse](docs/QuoteResponse.md)
- [RegisterShippingDestinationRequest](docs/RegisterShippingDestinationRequest.md)
- [ResponseFormat](docs/ResponseFormat.md)
- [Role](docs/Role.md)
- [ServiceAccountListResponse](docs/ServiceAccountListResponse.md)
- [ServiceAccountResponse](docs/ServiceAccountResponse.md)
- [ShippingAvailabilityResponse](docs/ShippingAvailabilityResponse.md)
- [ShippingDestinationResponse](docs/ShippingDestinationResponse.md)
- [SignInWithPlatformRequest](docs/SignInWithPlatformRequest.md)
- [SignInWithPlatformResponse](docs/SignInWithPlatformResponse.md)
- [SoftwareDeliveryResponse](docs/SoftwareDeliveryResponse.md)
- [TenantMappingResponse](docs/TenantMappingResponse.md)
- [Text](docs/Text.md)
- [Thinking](docs/Thinking.md)
- [Tool](docs/Tool.md)
- [ToolCall](docs/ToolCall.md)
- [ToolCallArgs](docs/ToolCallArgs.md)
- [ToolCallPending](docs/ToolCallPending.md)
- [ToolCallResponse](docs/ToolCallResponse.md)
- [ToolChoice](docs/ToolChoice.md)
- [ToolResult](docs/ToolResult.md)
- [ToolSchema](docs/ToolSchema.md)
- [UpdateChatroomRequest](docs/UpdateChatroomRequest.md)
- [UpdateChatroomResponse](docs/UpdateChatroomResponse.md)
- [UpdateProductRequest](docs/UpdateProductRequest.md)
- [UpdateProductVariationRequest](docs/UpdateProductVariationRequest.md)
- [UpdateServiceAccountRequest](docs/UpdateServiceAccountRequest.md)
- [Usage](docs/Usage.md)
- [User](docs/User.md)
- [UserListResponse](docs/UserListResponse.md)
- [UserMessage](docs/UserMessage.md)
- [UserResponse](docs/UserResponse.md)
- [VerifyRequest](docs/VerifyRequest.md)
- [VerifyResponse](docs/VerifyResponse.md)

### Authorization

Endpoints do not require authorization.


## About

This TypeScript SDK client supports the [Fetch API](https://fetch.spec.whatwg.org/)
and is automatically generated by the
[OpenAPI Generator](https://openapi-generator.tech) project:

- API version: `0.48.0`
- Package version: `0.1.0`
- Generator version: `7.20.0`
- Build package: `org.openapitools.codegen.languages.TypeScriptFetchClientCodegen`

The generated npm module supports the following:

- Environments
  * Node.js
  * Webpack
  * Browserify
- Language levels
  * ES5 - you must have a Promises/A+ library installed
  * ES6
- Module systems
  * CommonJS
  * ES6 module system


## Development

### Building

To build the TypeScript source code, you need to have Node.js and npm installed.
After cloning the repository, navigate to the project directory and run:

```bash
npm install
npm run build
```

### Publishing

Once you've built the package, you can publish it to npm:

```bash
npm publish
```

## License

[]()
