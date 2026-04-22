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

  // Checkout types
  CheckoutInput,

  // GraphQL types
  GraphQLError,
  GraphQLResponse,
  GraphQLRequestOptions,

  // Client config
  StorekitClientConfig,
} from "./types.js";

// Enums
export { ProductStatus, ProductVariantStatus, ConsumerOrderStatus } from "./types.js";
