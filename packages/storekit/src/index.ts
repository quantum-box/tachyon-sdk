/**
 * @quantum-box/storekit
 * Type-safe TypeScript SDK for bakuure commerce API
 */

// Main Client
export { StorekitClient } from "./client.js";

// Core Client
export {
  GraphQLClient,
  GraphQLClientError,
} from "./graphql-client.js";

// Types
export type {
  // Product types
  Product,
  ProductVariant,
  ProductPageInfo,
  ProductConnection,
  ProductsInput,

  // Storefront types
  StorefrontProduct,
  StorefrontCategory,
  StorefrontProductsInput,
  StorefrontProductConnection,
  CouponValidation,

  // Cart types
  Cart,
  CartItem,
  CreateCartInput,
  AddCartItemInput,
  UpdateCartItemInput,

  // Order types
  ConsumerOrder,
  ConsumerOrderItem,
  ConsumerOrderList,
  ConsumerOrdersInput,
  OrderList,
  OrderListInput,
  OrderLookupInput,
  OrderStatus,
  RefundResult,

  // Payment types
  StoreKitPaymentProvider,
  StoreKitPaymentKind,
  StoreKitPaymentStatus,
  StoreKitPaymentReference,
  StoreKitPaymentCustomer,
  StoreKitPaymentCreateInput,
  StoreKitPayment,
  StoreKitRefund,
  StoreKitRefundInput,
  StoreKitPaymentGetInput,
  StoreKitPaymentCancelInput,
  StoreKitWebhookVerifyInput,
  StoreKitVerifiedWebhook,
  StoreKitPaymentEventType,
  StoreKitPaymentEvent,
  StoreKitPaymentProviderAdapter,
  StoreKitPaymentProviderRegistration,
  StoreKitPaymentLogger,
  StoreKitPaymentsConfig,

  // Checkout types
  CheckoutInput,

  // Auth types
  UserProfile,
  AuthResult,
  SignInWithPlatformInput,
  CreateUserInput,
  UpdateProfileInput,

  // Inventory types
  StockInfo,
  ProductStock,

  // GraphQL types
  GraphQLError,
  GraphQLResponse,
  GraphQLRequestOptions,

  // Client config
  StorekitClientConfig,
} from "./types.js";

// Enums
export {
  ProductStatus,
  ProductVariantStatus,
  ConsumerOrderStatus,
  StorefrontProductSortOrder,
  UserRole,
} from "./types.js";

// Operations
export { AuthOperations } from "./operations/auth.js";
export { InventoryOperations } from "./operations/inventory.js";
export { OrdersOperations } from "./operations/orders.js";
export { StorefrontOperations } from "./operations/storefront.js";

// Payments
export {
  STOREKIT_PAYMENT_STATE_MACHINE,
  StoreKitPaymentProviderRegistry,
  StoreKitPaymentProviderSelectionError,
  StoreKitPaymentStateTransitionError,
  transitionStoreKitPaymentStatus,
} from "./payment-registry.js";

export {
  StoreKitSquareAdapter,
  StoreKitSquareAdapterError,
  StoreKitSquarePaymentNotFoundError,
  StoreKitSquareTenantMismatchError,
  StoreKitSquareWebhookSignatureError,
  verifySquareSignature,
} from "./adapters/index.js";
export type {
  StoreKitSquareAdapterConfig,
  StoreKitSquareClient,
  StoreKitSquarePaymentStore,
} from "./adapters/index.js";
