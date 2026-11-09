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
// Storefront Types
// ============================================================================

export interface StorefrontProduct {
  id: string;
  name: string;
  description: string | null;
  kind: string;
  listPrice: number;
  billingCycle: string;
  publicationName: string | null;
  publicationDescription: string | null;
  imageIds: string[];
  categoryId: string | null;
  weightGrams: number | null;
}

export interface StorefrontCategory {
  id: string;
  name: string;
  slug: string;
}

export enum StorefrontProductSortOrder {
  PRICE_ASC = "PRICE_ASC",
  PRICE_DESC = "PRICE_DESC",
  NAME_ASC = "NAME_ASC",
}

export interface StorefrontProductsInput {
  categoryId?: string;
  search?: string;
  priceMin?: number;
  priceMax?: number;
  sort?: StorefrontProductSortOrder;
  inStock?: boolean;
  limit?: number;
  offset?: number;
}

export interface StorefrontProductConnection {
  items: StorefrontProduct[];
  limit: number;
  offset: number;
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
  checkoutUrl: string | null;
  pickupRequestedAt: string | null;
  pickupDeadline: string | null;
  confirmedAt: string | null;
  shippedAt: string | null;
  deliveredAt: string | null;
  cancelledAt: string | null;
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

export interface OrderLookupInput {
  phone: string;
  lastDigits: string;
}

export interface OrderListInput {
  limit?: number;
  cursor?: string;
}

export interface OrderList {
  items: ConsumerOrder[];
  limit: number;
  cursor: string | null;
  hasNextPage: boolean;
}

export interface ConsumerOrdersInput {
  userId?: string;
  sessionId?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

export type OrderStatus = ConsumerOrderStatus;

// Scaffold type — shape subject to change pending PLT-723 approval
export interface RefundResult {
  orderId: string;
  refundedAmount: number;
  currency: string;
  status: string;
}

// ============================================================================
// Payment Types
// ============================================================================

export type StoreKitPaymentProvider = "square" | "stripe";

export type StoreKitPaymentKind =
  | "reservation_cancellation_fee"
  | "invoice_payment"
  | "order_checkout";

export type StoreKitPaymentStatus =
  | "pending"
  | "requires_action"
  | "paid"
  | "cancelled"
  | "refunded"
  | "failed";

export interface StoreKitPaymentReference {
  invoiceId?: string;
  reservationId?: string;
  fieldPaymentIntentId?: string;
  orderId?: string;
}

export interface StoreKitPaymentCustomer {
  email?: string;
  name?: string;
}

export interface StoreKitPaymentCreateInput {
  tenantId: string;
  kind: StoreKitPaymentKind;
  amountMinor: number;
  currency: string;
  reference: StoreKitPaymentReference;
  customer?: StoreKitPaymentCustomer;
  successUrl?: string;
  cancelUrl?: string;
  idempotencyKey: string;
  preferredProviders?: StoreKitPaymentProvider[];
}

export interface StoreKitPayment {
  id: string;
  tenantId: string;
  provider: StoreKitPaymentProvider;
  kind: StoreKitPaymentKind;
  status: StoreKitPaymentStatus;
  amountMinor: number;
  currency: string;
  checkoutUrl?: string;
  providerPaymentId: string;
  providerReference?: string;
  idempotencyKey: string;
  createdAt: string;
  updatedAt: string;
}

export interface StoreKitRefund {
  id: string;
  tenantId: string;
  paymentId: string;
  provider: StoreKitPaymentProvider;
  amountMinor: number;
  currency: string;
  status: "pending" | "succeeded" | "failed";
  reason?: string;
  idempotencyKey: string;
  providerRefundId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface StoreKitRefundInput {
  tenantId: string;
  paymentId: string;
  amountMinor?: number;
  reason?: string;
  idempotencyKey: string;
}

export interface StoreKitPaymentGetInput {
  tenantId: string;
  paymentId: string;
}

export interface StoreKitPaymentCancelInput {
  tenantId: string;
  paymentId: string;
  idempotencyKey: string;
}

export interface StoreKitWebhookVerifyInput {
  tenantId: string;
  rawBody: string;
  headers: Record<string, string>;
}

export interface StoreKitVerifiedWebhook {
  provider: StoreKitPaymentProvider;
  tenantId: string;
  eventId: string;
  rawEventType: string;
  receivedAt: string;
  payment?: StoreKitPayment;
  rawEvent?: unknown;
}

export type StoreKitPaymentEventType =
  | "payment_paid"
  | "payment_cancelled"
  | "payment_refunded"
  | "payment_partially_refunded"
  | "payment_failed"
  | "payment_dispute_opened"
  | "payment_dispute_closed"
  | "payment_expired"
  | "payment_unknown";

export interface StoreKitPaymentEvent {
  provider: StoreKitPaymentProvider;
  eventId: string;
  eventType: StoreKitPaymentEventType;
  payment?: StoreKitPayment;
  receivedAt: string;
  rawEventType?: string;
}

export interface StoreKitPaymentProviderAdapter {
  readonly provider: StoreKitPaymentProvider;

  createPayment(input: StoreKitPaymentCreateInput): Promise<StoreKitPayment>;
  getPayment(input: StoreKitPaymentGetInput): Promise<StoreKitPayment>;
  cancelPayment(input: StoreKitPaymentCancelInput): Promise<StoreKitPayment>;
  refundPayment(input: StoreKitRefundInput): Promise<StoreKitPayment>;
  verifyWebhook(input: StoreKitWebhookVerifyInput): Promise<StoreKitVerifiedWebhook>;
  normalizeWebhook(input: StoreKitVerifiedWebhook): Promise<StoreKitPaymentEvent>;
}

export interface StoreKitPaymentProviderRegistration {
  adapter: StoreKitPaymentProviderAdapter;
  isConfigured?: boolean;
  unavailableReason?: string;
}

export interface StoreKitPaymentLogger {
  warn(message: string, context?: Record<string, unknown>): void;
}

export interface StoreKitPaymentsConfig {
  providerPriority: StoreKitPaymentProvider[];
  providers?: StoreKitPaymentProviderRegistration[];
  logger?: StoreKitPaymentLogger;
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
  customerEmail?: string;
  pickupRequestedAt?: string;
  successUrl?: string;
  cancelUrl?: string;
}

export interface CouponValidation {
  id: string;
  code: string;
  discountType: string;
  discountValue: number;
  currency: string;
  isActive: boolean;
  discountAmount: number | null;
}

// ============================================================================
// Auth Types
// ============================================================================

export enum UserRole {
  OWNER = "OWNER",
  MANAGER = "MANAGER",
  GENERAL = "GENERAL",
  STORE = "STORE",
}

export interface UserProfile {
  id: string;
  email: string | null;
  name: string | null;
  username: string | null;
  emailVerified: string | null;
  image: string | null;
  role: UserRole;
  tenantIdList: string[];
  createdAt: string;
  updatedAt: string;
}

export interface AuthResult {
  user: UserProfile;
}

export interface SignInWithPlatformInput {
  platformId: string;
  accessToken: string;
  allowSignUp?: boolean;
}

export interface CreateUserInput {
  operatorId: string;
  username: string;
  email: string;
  image?: string;
  role?: string;
  password?: string;
}

export interface UpdateProfileInput {
  id: string;
  email?: string;
  name?: string;
}

// ============================================================================
// Inventory Types
// ============================================================================

export interface StockInfo {
  id: string;
  productId: string;
  quantityOnHand: number;
  quantityReserved: number;
  quantityAvailable: number;
  lowStockThreshold: number;
  trackInventory: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ProductStock extends StockInfo {}

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
  payments?: StoreKitPaymentsConfig;
}
