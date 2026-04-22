/**
 * Main StorekitClient
 * Type-safe TypeScript SDK for bakuure commerce API
 */

import { GraphQLClient } from "./graphql-client.js";
import { ProductsOperations } from "./operations/products.js";
import { OrdersOperations } from "./operations/orders.js";
import { CartOperations } from "./operations/cart.js";
import type { StorekitClientConfig } from "./types.js";

export class StorekitClient {
  public readonly products: ProductsOperations;
  public readonly orders: OrdersOperations;
  public readonly cart: CartOperations;

  private readonly graphqlClient: GraphQLClient;

  constructor(config: StorekitClientConfig) {
    this.graphqlClient = new GraphQLClient(config.baseUrl, {
      apiKey: config.apiKey,
      bearerToken: config.bearerToken,
      headers: config.headers,
    });

    this.products = new ProductsOperations(this.graphqlClient);
    this.orders = new OrdersOperations(this.graphqlClient);
    this.cart = new CartOperations(this.graphqlClient);
  }
}
