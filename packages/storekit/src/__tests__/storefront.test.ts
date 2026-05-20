/**
 * Unit tests for StorefrontOperations
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { StorefrontOperations } from "../operations/storefront.js";
import { GraphQLClientError } from "../graphql-client.js";
import { StorefrontProductSortOrder } from "../types.js";
import type {
  CouponValidation,
  StockInfo,
  StorefrontCategory,
  StorefrontProduct,
  StorefrontProductConnection,
} from "../types.js";

const makeStorefrontProduct = (
  overrides: Partial<StorefrontProduct> = {},
): StorefrontProduct => ({
  id: "prod-1",
  name: "Published Product",
  description: "Visible on the storefront",
  kind: "physical",
  listPrice: 1000,
  billingCycle: "one_time",
  publicationName: "Published Product",
  publicationDescription: "Storefront description",
  imageIds: ["image-1"],
  categoryId: "cat-1",
  weightGrams: 250,
  ...overrides,
});

const makeConnection = (
  overrides: Partial<StorefrontProductConnection> = {},
): StorefrontProductConnection => ({
  items: [makeStorefrontProduct()],
  limit: 25,
  offset: 0,
  ...overrides,
});

const makeStock = (overrides: Partial<StockInfo> = {}): StockInfo => ({
  id: "stock-1",
  productId: "prod-1",
  quantityOnHand: 10,
  quantityReserved: 2,
  quantityAvailable: 8,
  lowStockThreshold: 3,
  trackInventory: true,
  createdAt: "2026-04-22T00:00:00Z",
  updatedAt: "2026-04-22T00:00:00Z",
  ...overrides,
});

const makeCategory = (
  overrides: Partial<StorefrontCategory> = {},
): StorefrontCategory => ({
  id: "cat-1",
  name: "Coffee",
  slug: "coffee",
  ...overrides,
});

const makeCouponValidation = (
  overrides: Partial<CouponValidation> = {},
): CouponValidation => ({
  id: "coupon-1",
  code: "SAVE10",
  discountType: "percentage",
  discountValue: 10,
  currency: "USD",
  isActive: true,
  discountAmount: 100,
  ...overrides,
});

describe("StorefrontOperations", () => {
  let mockClient: { query: ReturnType<typeof vi.fn> };
  let storefront: StorefrontOperations;

  beforeEach(() => {
    mockClient = { query: vi.fn() };
    storefront = new StorefrontOperations(mockClient as any);
  });

  describe("list", () => {
    it("should default filters and pagination when no input is supplied", async () => {
      const connection = makeConnection();
      mockClient.query.mockResolvedValueOnce({ storefrontProducts: connection });

      const result = await storefront.list();

      expect(result).toEqual(connection);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("storefrontProducts"),
        {
          categoryId: null,
          search: null,
          priceMin: null,
          priceMax: null,
          sort: null,
          inStock: null,
          limit: 25,
          offset: 0,
        },
      );
    });

    it("should forward storefront product filters", async () => {
      const connection = makeConnection({ limit: 10, offset: 20 });
      mockClient.query.mockResolvedValueOnce({ storefrontProducts: connection });

      const result = await storefront.list({
        categoryId: "cat-1",
        search: "coffee",
        priceMin: 100,
        priceMax: 2000,
        sort: StorefrontProductSortOrder.PRICE_ASC,
        inStock: true,
        limit: 10,
        offset: 20,
      });

      expect(result).toEqual(connection);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.any(String),
        {
          categoryId: "cat-1",
          search: "coffee",
          priceMin: 100,
          priceMax: 2000,
          sort: StorefrontProductSortOrder.PRICE_ASC,
          inStock: true,
          limit: 10,
          offset: 20,
        },
      );
    });
  });

  describe("get", () => {
    it("should return a storefront product by id", async () => {
      const product = makeStorefrontProduct();
      mockClient.query.mockResolvedValueOnce({ storefrontProduct: product });

      const result = await storefront.get("prod-1");

      expect(result).toEqual(product);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("storefrontProduct"),
        { productId: "prod-1" },
      );
    });

    it("should propagate GraphQLClientError for a not-found product", async () => {
      const err = new GraphQLClientError("GraphQL query failed", [
        { message: "Product not found" },
      ]);
      mockClient.query.mockRejectedValueOnce(err);

      await expect(storefront.get("does-not-exist")).rejects.toBeInstanceOf(
        GraphQLClientError,
      );
    });
  });

  describe("getWithStock", () => {
    it("should return a storefront product with stock info", async () => {
      const product = makeStorefrontProduct();
      const stock = makeStock();
      mockClient.query.mockResolvedValueOnce({
        storefrontProduct: product,
        productStock: stock,
      });

      const result = await storefront.getWithStock("prod-1");

      expect(result).toEqual({ product, stock });
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("productStock"),
        { productId: "prod-1" },
      );
    });
  });

  describe("categories", () => {
    it("should return storefront categories", async () => {
      const categories = [makeCategory()];
      mockClient.query.mockResolvedValueOnce({ storefrontCategories: categories });

      const result = await storefront.categories();

      expect(result).toEqual(categories);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("storefrontCategories"),
      );
    });
  });

  describe("validateCoupon", () => {
    it("should validate a coupon with subtotal", async () => {
      const validation = makeCouponValidation();
      mockClient.query.mockResolvedValueOnce({ validateCoupon: validation });

      const result = await storefront.validateCoupon("SAVE10", 1000);

      expect(result).toEqual(validation);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("validateCoupon"),
        { code: "SAVE10", subtotalNanodollar: 1000 },
      );
    });

    it("should pass null subtotal when omitted", async () => {
      const validation = makeCouponValidation({ discountAmount: null });
      mockClient.query.mockResolvedValueOnce({ validateCoupon: validation });

      await storefront.validateCoupon("SAVE10");

      expect(mockClient.query).toHaveBeenCalledWith(
        expect.any(String),
        { code: "SAVE10", subtotalNanodollar: null },
      );
    });
  });
});
