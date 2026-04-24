/**
 * Unit tests for ProductsOperations
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { ProductsOperations } from "../operations/products.js";
import { GraphQLClientError } from "../graphql-client.js";
import { ProductStatus, ProductVariantStatus } from "../types.js";
import type { Product, ProductConnection } from "../types.js";

const makeProduct = (overrides: Partial<Product> = {}): Product => ({
  id: "prod-1",
  tenantId: "tenant-1",
  name: "Test Product",
  description: "A product used for testing",
  status: ProductStatus.ACTIVE,
  skuCode: "SKU-001",
  janCode: null,
  upcCode: null,
  kind: "physical",
  billingCycle: "one_time",
  listPrice: 1000,
  publicationStatus: "published",
  publicationName: "Test Product",
  publicationDescription: "Public description",
  imageFileIds: [],
  imageStorageKeys: [],
  createdAt: "2026-04-22T00:00:00Z",
  updatedAt: "2026-04-22T00:00:00Z",
  variants: [
    {
      id: "variant-1",
      productId: "prod-1",
      tenantId: "tenant-1",
      code: "VAR-001",
      name: "Default",
      status: ProductVariantStatus.ACTIVE,
      metadata: {},
      createdAt: "2026-04-22T00:00:00Z",
      updatedAt: "2026-04-22T00:00:00Z",
    },
  ],
  ...overrides,
});

const makeConnection = (
  overrides: Partial<ProductConnection> = {},
): ProductConnection => ({
  items: [makeProduct()],
  totalCount: 1,
  pageInfo: { limit: 25, offset: 0, hasNextPage: false },
  ...overrides,
});

describe("ProductsOperations", () => {
  let mockClient: { query: ReturnType<typeof vi.fn>; mutate: ReturnType<typeof vi.fn> };
  let products: ProductsOperations;

  beforeEach(() => {
    mockClient = { query: vi.fn(), mutate: vi.fn() };
    products = new ProductsOperations(mockClient as any);
  });

  describe("list", () => {
    it("should default to limit=25 offset=0 when no input is supplied", async () => {
      const connection = makeConnection();
      mockClient.query.mockResolvedValueOnce({ products: connection });

      const result = await products.list();

      expect(result).toEqual(connection);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("products"),
        { limit: 25, offset: 0 },
      );
    });

    it("should forward limit and offset variables for pagination", async () => {
      const connection = makeConnection({
        pageInfo: { limit: 5, offset: 10, hasNextPage: true },
      });
      mockClient.query.mockResolvedValueOnce({ products: connection });

      const result = await products.list({ limit: 5, offset: 10 });

      expect(result).toEqual(connection);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.any(String),
        { limit: 5, offset: 10 },
      );
    });
  });

  describe("get", () => {
    it("should return a single product by id", async () => {
      const product = makeProduct();
      mockClient.query.mockResolvedValueOnce({ product });

      const result = await products.get("prod-1");

      expect(result).toEqual(product);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("product"),
        { id: "prod-1" },
      );
    });

    it("should propagate GraphQLClientError for a not-found id", async () => {
      const err = new GraphQLClientError("GraphQL query failed", [
        { message: "Product not found" },
      ]);
      mockClient.query.mockRejectedValueOnce(err);

      await expect(products.get("does-not-exist")).rejects.toBeInstanceOf(
        GraphQLClientError,
      );
    });

    it("should propagate a generic network Error", async () => {
      mockClient.query.mockRejectedValueOnce(new Error("network failure"));

      await expect(products.get("prod-1")).rejects.toThrow("network failure");
    });
  });
});
