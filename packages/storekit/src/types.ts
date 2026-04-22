/**
 * Core types for the StoreKit SDK
 * Based on bakuure GraphQL schema
 */

// ============================================================================
// Product Types
// ============================================================================

export interface Product {
  id: string;
  tenantId: string;
  name: string;
  description: string | null;
  status: ProductStatus;
  skuCode: string | null;
  janCode: string | null;
  upcCode: string | null;
  kind: string;
  billingCycle: string;
  listPrice: number;
  publicationStatus: string;
  publicationName: string | null;
  publicationDescription: string | null;
  imageFileIds: string[];
  imageStorageKeys: string[];
  createdAt: string;
  updatedAt: string;
  variants: ProductVariant[];
}

export interface ProductVariant {
  id: string;
  productId: string;
  tenantId: string;
  code: string;
  name: string;
  status: ProductVariantStatus;
  metadata: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

export enum ProductStatus {
  DRAFT = "DRAFT",
  ACTIVE = "ACTIVE",
  ARCHIVED = "ARCHIVED",
}

export enum ProductVariantStatus {
  DRAFT = "DRAFT",
  ACTIVE = "ACTIVE",
  ARCHIVED = "ARCHIVED",
}

export interface ProductPageInfo {
  limit: number;
  offset: number;
  hasNextPage: boolean;
}

export interface ProductConnection {
  items: Product[];
  totalCount: number;
  pageInfo: ProductPageInfo;
}

export interface ProductsInput {
  limit?: number;
  offset?: number;
}

// ============================================================================
// Cart Types
// ============================================================================

export interface Cart {
  id: string;
  tenantId: string;
  userId: string | null;
  sessionId: string | null;
  status: string;
  items: CartItem[];
  expiresAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CartItem {
  id: string;
  productId: string;
  quantity: number;
  unitPriceNanodollar: string;
}

export interface CreateCartInput {
  userId?: string;
  sessionId?: string;
}

export interface AddCartItemInput {
  productId: string;
  quantity: number;
}

export interface UpdateCartItemInput {
  quantity: number;
}

// ============================================================================
// Order Types
// ============================================================================

export interface ConsumerOrder {
  id: string;
  tenantId: string;
  cartId: string | null;
  userId: string | null;
  sessionId: string | null;
  status: ConsumerOrderStatus;
  fulfillmentMethod: string | null;
  paymentMethod: string | null;
  shippingName: string | null;
  shippingAddress: string | null;
  shippingPhone: string | null;
  subtotalNanodollar: string;
  discountNanodollar: string;
  shippingFeeNanodollar: string;
  totalNanodollar: string;
  items: ConsumerOrderItem[];
  confirmedAt: string | null;
  shippedAt: string | null;
  deliveredAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface ConsumerOrderItem {
  id: string;
  productId: string;
  productName: string;
  quantity: number;
  unitPriceNanodollar: string;
  subtotalNanodollar: string;
}

export enum ConsumerOrderStatus {
  PENDING = "pending",
  CONFIRMED = "confirmed",
  SHIPPED = "shipped",
  DELIVERED = "delivered",
  CANCELLED = "cancelled",
}

export interface ConsumerOrderList {
  items: ConsumerOrder[];
  limit: number;
  offset: number;
}

export interface ConsumerOrdersInput {
  userId?: string;
  sessionId?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

// ============================================================================
// Checkout Types
// ============================================================================

export interface CheckoutInput {
  cartId: string;
  shippingName?: string;
  shippingAddress?: string;
  shippingPhone?: string;
  fulfillmentMethod?: "pickup" | "delivery";
  paymentMethod?: "in_store" | "online";
  couponCode?: string;
}

// ============================================================================
// GraphQL Types
// ============================================================================

export interface GraphQLError {
  message: string;
  locations?: Array<{ line: number; column: number }>;
  path?: Array<string | number>;
  extensions?: Record<string, unknown>;
}

export interface GraphQLResponse<T> {
  data?: T;
  errors?: GraphQLError[];
}

export interface GraphQLRequestOptions {
  apiKey?: string;
  bearerToken?: string;
  headers?: Record<string, string>;
}

// ============================================================================
// Client Config
// ============================================================================

export interface StorekitClientConfig {
  baseUrl: string;
  apiKey?: string;
  bearerToken?: string;
  headers?: Record<string, string>;
}
