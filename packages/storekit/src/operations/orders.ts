/**
 * Order operations
 */

import type {
  ConsumerOrder,
  ConsumerOrderList,
  ConsumerOrdersInput,
  OrderStatus,
  RefundResult,
} from "../types.js";
import { ConsumerOrderStatus } from "../types.js";

interface GraphQLClient {
  query<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
  mutate<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
}

// GraphQL Queries
const GET_CONSUMER_ORDERS = `
  query ConsumerOrders(
    $userId: String
    $sessionId: String
    $status: String
    $limit: Int = 20
    $offset: Int = 0
  ) {
    consumerOrders(
      userId: $userId
      sessionId: $sessionId
      status: $status
      limit: $limit
      offset: $offset
    ) {
      items {
        id
        tenantId
        cartId
        userId
        sessionId
        status
        fulfillmentMethod
        paymentMethod
        shippingName
        shippingAddress
        shippingPhone
        subtotalNanodollar
        discountNanodollar
        shippingFeeNanodollar
        totalNanodollar
        items {
          id
          productId
          productName
          quantity
          unitPriceNanodollar
          subtotalNanodollar
        }
        confirmedAt
        shippedAt
        deliveredAt
        createdAt
        updatedAt
      }
      limit
      offset
    }
  }
`;

const GET_CONSUMER_ORDER = `
  query ConsumerOrder($orderId: ID!) {
    consumerOrder(orderId: $orderId) {
      id
      tenantId
      cartId
      userId
      sessionId
      status
      fulfillmentMethod
      paymentMethod
      shippingName
      shippingAddress
      shippingPhone
      subtotalNanodollar
      discountNanodollar
      shippingFeeNanodollar
      totalNanodollar
      items {
        id
        productId
        productName
        quantity
        unitPriceNanodollar
        subtotalNanodollar
      }
      confirmedAt
      shippedAt
      deliveredAt
      createdAt
      updatedAt
    }
  }
`;

const ORDER_FIELDS = `
  id tenantId cartId userId sessionId status fulfillmentMethod paymentMethod
  shippingName shippingAddress shippingPhone subtotalNanodollar discountNanodollar
  shippingFeeNanodollar totalNanodollar
  items { id productId productName quantity unitPriceNanodollar subtotalNanodollar }
  confirmedAt shippedAt deliveredAt createdAt updatedAt
`;

const CONFIRM_ORDER = `mutation ConfirmOrder($orderId: ID!) { confirmOrder(orderId: $orderId) { ${ORDER_FIELDS} } }`;
const SHIP_ORDER = `mutation ShipOrder($orderId: ID!) { shipOrder(orderId: $orderId) { ${ORDER_FIELDS} } }`;
const DELIVER_ORDER = `mutation DeliverOrder($orderId: ID!) { deliverOrder(orderId: $orderId) { ${ORDER_FIELDS} } }`;

// cancelOrder returns Boolean!, so we re-fetch the order after cancellation
const CANCEL_ORDER = `mutation CancelOrder($orderId: ID!) { cancelOrder(orderId: $orderId) }`;

export class OrdersOperations {
  private readonly client: GraphQLClient;

  constructor(client: GraphQLClient) {
    this.client = client;
  }

  /**
   * Get a list of consumer orders
   */
  async list(input: ConsumerOrdersInput = {}): Promise<ConsumerOrderList> {
    const response = await this.client.query<{
      consumerOrders: ConsumerOrderList;
    }>(GET_CONSUMER_ORDERS, {
      userId: input.userId ?? null,
      sessionId: input.sessionId ?? null,
      status: input.status ?? null,
      limit: input.limit ?? 20,
      offset: input.offset ?? 0,
    });
    return response.consumerOrders;
  }

  /**
   * Get a single order by ID
   */
  async get(orderId: string): Promise<ConsumerOrder> {
    const response = await this.client.query<{
      consumerOrder: ConsumerOrder;
    }>(GET_CONSUMER_ORDER, { orderId });
    return response.consumerOrder;
  }

  /**
   * Update order status via the appropriate state-transition mutation.
   * The bakuure API exposes per-transition mutations rather than a generic
   * updateOrderStatus — this method maps ConsumerOrderStatus to the correct one.
   * TODO: replace with a unified updateOrderStatus mutation once the backend adds it.
   */
  async updateStatus(orderId: string, status: OrderStatus): Promise<ConsumerOrder> {
    switch (status) {
      case ConsumerOrderStatus.CONFIRMED: {
        const r = await this.client.mutate<{ confirmOrder: ConsumerOrder }>(
          CONFIRM_ORDER,
          { orderId },
        );
        return r.confirmOrder;
      }
      case ConsumerOrderStatus.SHIPPED: {
        const r = await this.client.mutate<{ shipOrder: ConsumerOrder }>(
          SHIP_ORDER,
          { orderId },
        );
        return r.shipOrder;
      }
      case ConsumerOrderStatus.DELIVERED: {
        const r = await this.client.mutate<{ deliverOrder: ConsumerOrder }>(
          DELIVER_ORDER,
          { orderId },
        );
        return r.deliverOrder;
      }
      case ConsumerOrderStatus.CANCELLED:
        return this.cancel(orderId);
      default:
        throw new Error(
          `updateStatus: no backend mutation for status "${status}"`,
        );
    }
  }

  /**
   * Cancel an order.
   * The reason parameter is captured for future use but is not yet forwarded
   * to the backend (cancelOrder accepts only orderId).
   * TODO: pass reason once bakuure API supports it.
   */
  async cancel(orderId: string, _reason?: string): Promise<ConsumerOrder> {
    await this.client.mutate<{ cancelOrder: boolean }>(CANCEL_ORDER, {
      orderId,
    });
    return this.get(orderId);
  }

  /**
   * Refund an order.
   * @throws Not implemented — requires PLT-723 CEO approval before the
   *   refund backend and this method can be completed.
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async refund(_orderId: string, _amount?: number): Promise<RefundResult> {
    throw new Error("Not implemented: requires PLT-723 approval");
  }
}
