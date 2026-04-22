/**
 * Order operations
 */

import type {
  ConsumerOrder,
  ConsumerOrderList,
  ConsumerOrdersInput,
} from "../types.js";

interface GraphQLClient {
  query<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
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
}
