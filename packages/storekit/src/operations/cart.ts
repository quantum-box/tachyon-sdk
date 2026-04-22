/**
 * Cart operations
 */

import type {
  Cart,
  CreateCartInput,
  AddCartItemInput,
  UpdateCartItemInput,
  ConsumerOrder,
  CheckoutInput,
} from "../types.js";

interface GraphQLClient {
  query<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
  mutate<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
}

// GraphQL Queries
const GET_CART = `
  query Cart($cartId: ID!) {
    cart(cartId: $cartId) {
      id
      tenantId
      userId
      sessionId
      status
      items {
        id
        productId
        quantity
        unitPriceNanodollar
      }
      expiresAt
      createdAt
      updatedAt
    }
  }
`;

// GraphQL Mutations
const CREATE_CART = `
  mutation CreateCart($input: CreateCartInput!) {
    createCart(input: $input) {
      id
      tenantId
      userId
      sessionId
      status
      items {
        id
        productId
        quantity
        unitPriceNanodollar
      }
      expiresAt
      createdAt
      updatedAt
    }
  }
`;

const ADD_CART_ITEM = `
  mutation AddCartItem($cartId: ID!, $input: AddCartItemInput!) {
    addCartItem(cartId: $cartId, input: $input) {
      id
      tenantId
      userId
      sessionId
      status
      items {
        id
        productId
        quantity
        unitPriceNanodollar
      }
      expiresAt
      createdAt
      updatedAt
    }
  }
`;

const UPDATE_CART_ITEM = `
  mutation UpdateCartItem(
    $cartId: ID!
    $itemId: ID!
    $input: UpdateCartItemInput!
  ) {
    updateCartItem(cartId: $cartId, itemId: $itemId, input: $input) {
      id
      tenantId
      userId
      sessionId
      status
      items {
        id
        productId
        quantity
        unitPriceNanodollar
      }
      expiresAt
      createdAt
      updatedAt
    }
  }
`;

const REMOVE_CART_ITEM = `
  mutation RemoveCartItem($cartId: ID!, $itemId: ID!) {
    removeCartItem(cartId: $cartId, itemId: $itemId)
  }
`;

const CLEAR_CART = `
  mutation ClearCart($cartId: ID!) {
    clearCart(cartId: $cartId)
  }
`;

const CHECKOUT = `
  mutation Checkout($input: CheckoutInput!) {
    checkout(input: $input) {
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

export class CartOperations {
  private readonly client: GraphQLClient;

  constructor(client: GraphQLClient) {
    this.client = client;
  }

  /**
   * Get a cart by ID
   */
  async get(cartId: string): Promise<Cart> {
    const response = await this.client.query<{ cart: Cart }>(GET_CART, {
      cartId,
    });
    return response.cart;
  }

  /**
   * Create a new cart
   */
  async create(input: CreateCartInput = {}): Promise<Cart> {
    const response = await this.client.mutate<{ createCart: Cart }>(
      CREATE_CART,
      { input },
    );
    return response.createCart;
  }

  /**
   * Add an item to the cart
   */
  async addItem(
    cartId: string,
    input: AddCartItemInput,
  ): Promise<Cart> {
    const response = await this.client.mutate<{ addCartItem: Cart }>(
      ADD_CART_ITEM,
      { cartId, input },
    );
    return response.addCartItem;
  }

  /**
   * Update an item in the cart
   */
  async updateItem(
    cartId: string,
    itemId: string,
    input: UpdateCartItemInput,
  ): Promise<Cart> {
    const response = await this.client.mutate<{ updateCartItem: Cart }>(
      UPDATE_CART_ITEM,
      { cartId, itemId, input },
    );
    return response.updateCartItem;
  }

  /**
   * Remove an item from the cart
   */
  async removeItem(cartId: string, itemId: string): Promise<boolean> {
    const response = await this.client.mutate<{ removeCartItem: boolean }>(
      REMOVE_CART_ITEM,
      { cartId, itemId },
    );
    return response.removeCartItem;
  }

  /**
   * Clear all items from the cart
   */
  async clear(cartId: string): Promise<boolean> {
    const response = await this.client.mutate<{ clearCart: boolean }>(
      CLEAR_CART,
      { cartId },
    );
    return response.clearCart;
  }

  /**
   * Checkout - convert cart to order
   */
  async checkout(input: CheckoutInput): Promise<ConsumerOrder> {
    const response = await this.client.mutate<{ checkout: ConsumerOrder }>(
      CHECKOUT,
      { input },
    );
    return response.checkout;
  }
}
