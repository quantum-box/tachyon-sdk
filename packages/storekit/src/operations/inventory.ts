/**
 * Inventory operations
 */

import type { StockInfo, ProductStock } from "../types.js";

interface GraphQLClient {
  query<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
  mutate<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
}

const STOCK_FIELDS = `
  id
  productId
  quantityOnHand
  quantityReserved
  quantityAvailable
  lowStockThreshold
  trackInventory
  createdAt
  updatedAt
`;

const GET_PRODUCT_STOCK = `
  query ProductStock($productId: ID!) {
    productStock(productId: $productId) {
      ${STOCK_FIELDS}
    }
  }
`;

const ADJUST_STOCK = `
  mutation AdjustStock($productId: ID!, $input: StockQuantityInput!) {
    adjustStock(productId: $productId, input: $input) {
      ${STOCK_FIELDS}
    }
  }
`;

export class InventoryOperations {
  private readonly client: GraphQLClient;

  constructor(client: GraphQLClient) {
    this.client = client;
  }

  async getStock(productId: string): Promise<StockInfo> {
    const response = await this.client.query<{ productStock: StockInfo }>(
      GET_PRODUCT_STOCK,
      { productId },
    );
    return response.productStock;
  }

  async updateStock(productId: string, quantity: number): Promise<StockInfo> {
    const response = await this.client.mutate<{ adjustStock: StockInfo }>(
      ADJUST_STOCK,
      { productId, input: { quantity } },
    );
    return response.adjustStock;
  }

  // TODO: bakuure-api does not yet expose a low-stock list query.
  // This method requires a future `lowStockProducts(threshold: Int)` query.
  async listLowStock(_threshold?: number): Promise<ProductStock[]> {
    throw new Error(
      "listLowStock is not yet supported: bakuure-api has no low-stock list query. " +
        "Track progress in PLT-774.",
    );
  }
}
