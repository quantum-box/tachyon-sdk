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
  OrderStatus,
  RefundResult,

  // Checkout types
  CheckoutInput,

  // Auth types
  UserProfile,
  AuthResult,
  SignInWithPlatformInput,
  CreateUserInput,
  UpdateProfileInput,

  // GraphQL types
  GraphQLError,
  GraphQLResponse,
  GraphQLRequestOptions,

  // Client config
  StorekitClientConfig,
} from "./types.js";

// Enums
export { ProductStatus, ProductVariantStatus, ConsumerOrderStatus, UserRole } from "./types.js";

// Operations
export { AuthOperations } from "./operations/auth.js";
