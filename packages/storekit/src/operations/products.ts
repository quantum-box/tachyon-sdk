/**
 * Product operations
 */

import type {
  Product,
  ProductConnection,
  ProductsInput,
} from "../types.js";

interface GraphQLClient {
  query<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
}

// GraphQL Queries
const GET_PRODUCTS = `
  query Products($limit: Int = 25, $offset: Int = 0) {
    products(limit: $limit, offset: $offset) {
      items {
        id
        tenantId
        name
        description
        status
        skuCode
        janCode
        upcCode
        kind
        billingCycle
        listPrice
        publicationStatus
        publicationName
        publicationDescription
        imageFileIds
        imageStorageKeys
        createdAt
        updatedAt
        variants {
          id
          productId
          tenantId
          code
          name
          status
          metadata
          createdAt
          updatedAt
        }
      }
      totalCount
      pageInfo {
        limit
        offset
        hasNextPage
      }
    }
  }
`;

const GET_PRODUCT = `
  query Product($id: ID!) {
    product(id: $id) {
      id
      tenantId
      name
      description
      status
      skuCode
      janCode
      upcCode
      kind
      billingCycle
      listPrice
      publicationStatus
      publicationName
      publicationDescription
      imageFileIds
      imageStorageKeys
      createdAt
      updatedAt
      variants {
        id
        productId
        tenantId
        code
        name
        status
        metadata
        createdAt
        updatedAt
      }
    }
  }
`;

export class ProductsOperations {
  private readonly client: GraphQLClient;

  constructor(client: GraphQLClient) {
    this.client = client;
  }

  /**
   * Get a list of products
   */
  async list(input: ProductsInput = {}): Promise<ProductConnection> {
    const response = await this.client.query<{ products: ProductConnection }>(
      GET_PRODUCTS,
      {
        limit: input.limit ?? 25,
        offset: input.offset ?? 0,
      },
    );
    return response.products;
  }

  /**
   * Get a single product by ID
   */
  async get(id: string): Promise<Product> {
    const response = await this.client.query<{ product: Product }>(
      GET_PRODUCT,
      { id },
    );
    return response.product;
  }
}
