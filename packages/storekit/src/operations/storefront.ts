/**
 * Storefront operations — public-facing product queries
 */

import type {
  StorefrontProduct,
  StorefrontProductConnection,
  StorefrontProductsInput,
  StorefrontCategory,
  StockInfo,
  CouponValidation,
} from "../types.js";

interface GraphQLClient {
  query<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
}

const GET_STOREFRONT_PRODUCTS = `
  query StorefrontProducts(
    $categoryId: String
    $search: String
    $priceMin: Int
    $priceMax: Int
    $sort: ProductSortOrder
    $inStock: Boolean
    $limit: Int = 25
    $offset: Int = 0
  ) {
    storefrontProducts(
      categoryId: $categoryId
      search: $search
      priceMin: $priceMin
      priceMax: $priceMax
      sort: $sort
      inStock: $inStock
      limit: $limit
      offset: $offset
    ) {
      items {
        id
        name
        description
        kind
        listPrice
        billingCycle
        publicationName
        publicationDescription
        imageIds
        categoryId
        weightGrams
      }
      limit
      offset
    }
  }
`;

const GET_STOREFRONT_PRODUCT = `
  query StorefrontProduct($productId: ID!) {
    storefrontProduct(productId: $productId) {
      id
      name
      description
      kind
      listPrice
      billingCycle
      publicationName
      publicationDescription
      imageIds
      categoryId
      weightGrams
    }
  }
`;

const GET_STOREFRONT_CATEGORIES = `
  query StorefrontCategories {
    storefrontCategories {
      id
      name
      slug
    }
  }
`;

const VALIDATE_COUPON = `
  query ValidateCoupon($code: String!, $subtotalNanodollar: Int) {
    validateCoupon(code: $code, subtotalNanodollar: $subtotalNanodollar) {
      id
      code
      discountType
      discountValue
      currency
      isActive
      discountAmount
    }
  }
`;

export class StorefrontOperations {
  private readonly client: GraphQLClient;

  constructor(client: GraphQLClient) {
    this.client = client;
  }

  async list(input: StorefrontProductsInput = {}): Promise<StorefrontProductConnection> {
    const response = await this.client.query<{
      storefrontProducts: StorefrontProductConnection;
    }>(GET_STOREFRONT_PRODUCTS, {
      categoryId: input.categoryId ?? null,
      search: input.search ?? null,
      priceMin: input.priceMin ?? null,
      priceMax: input.priceMax ?? null,
      sort: input.sort ?? null,
      inStock: input.inStock ?? null,
      limit: input.limit ?? 25,
      offset: input.offset ?? 0,
    });
    return response.storefrontProducts;
  }

  async get(productId: string): Promise<StorefrontProduct> {
    const response = await this.client.query<{
      storefrontProduct: StorefrontProduct;
    }>(GET_STOREFRONT_PRODUCT, { productId });
    return response.storefrontProduct;
  }

  async getWithStock(productId: string): Promise<{
    product: StorefrontProduct;
    stock: StockInfo;
  }> {
    const response = await this.client.query<{
      storefrontProduct: StorefrontProduct;
      productStock: StockInfo;
    }>(
      `query StorefrontProductWithStock($productId: ID!) {
        storefrontProduct(productId: $productId) {
          id name description kind listPrice billingCycle
          publicationName publicationDescription imageIds categoryId weightGrams
        }
        productStock(productId: $productId) {
          id productId quantityOnHand quantityReserved quantityAvailable
          lowStockThreshold trackInventory createdAt updatedAt
        }
      }`,
      { productId },
    );
    return {
      product: response.storefrontProduct,
      stock: response.productStock,
    };
  }

  async categories(): Promise<StorefrontCategory[]> {
    const response = await this.client.query<{
      storefrontCategories: StorefrontCategory[];
    }>(GET_STOREFRONT_CATEGORIES);
    return response.storefrontCategories;
  }

  async validateCoupon(
    code: string,
    subtotalNanodollar?: number,
  ): Promise<CouponValidation> {
    const response = await this.client.query<{
      validateCoupon: CouponValidation;
    }>(VALIDATE_COUPON, { code, subtotalNanodollar: subtotalNanodollar ?? null });
    return response.validateCoupon;
  }
}
