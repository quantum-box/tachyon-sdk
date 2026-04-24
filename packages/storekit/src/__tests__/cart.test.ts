/**
 * Unit tests for CartOperations
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { CartOperations } from "../operations/cart.js";
import { GraphQLClientError } from "../graphql-client.js";
import { ConsumerOrderStatus } from "../types.js";
import type { Cart, ConsumerOrder } from "../types.js";

const makeCart = (overrides: Partial<Cart> = {}): Cart => ({
  id: "cart-1",
  tenantId: "tenant-1",
  userId: null,
  sessionId: "session-1",
  status: "active",
  items: [],
  expiresAt: null,
  createdAt: "2026-04-22T00:00:00Z",
  updatedAt: "2026-04-22T00:00:00Z",
  ...overrides,
});

const makeOrder = (overrides: Partial<ConsumerOrder> = {}): ConsumerOrder => ({
  id: "order-1",
  tenantId: "tenant-1",
  cartId: "cart-1",
  userId: "user-1",
  sessionId: null,
  status: ConsumerOrderStatus.PENDING,
  fulfillmentMethod: "pickup",
  paymentMethod: "in_store",
  shippingName: null,
  shippingAddress: null,
  shippingPhone: null,
  subtotalNanodollar: "10000000000",
  discountNanodollar: "0",
  shippingFeeNanodollar: "0",
  totalNanodollar: "10000000000",
  items: [],
  confirmedAt: null,
  shippedAt: null,
  deliveredAt: null,
  createdAt: "2026-04-22T00:00:00Z",
  updatedAt: "2026-04-22T00:00:00Z",
  ...overrides,
});

describe("CartOperations", () => {
  let mockClient: { query: ReturnType<typeof vi.fn>; mutate: ReturnType<typeof vi.fn> };
  let cart: CartOperations;

  beforeEach(() => {
    mockClient = { query: vi.fn(), mutate: vi.fn() };
    cart = new CartOperations(mockClient as any);
  });

  describe("create", () => {
    it("should create a guest cart with sessionId", async () => {
      const guestCart = makeCart({ sessionId: "session-guest", userId: null });
      mockClient.mutate.mockResolvedValueOnce({ createCart: guestCart });

      const result = await cart.create({ sessionId: "session-guest" });

      expect(result).toEqual(guestCart);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("createCart"),
        { input: { sessionId: "session-guest" } },
      );
    });

    it("should create an authenticated cart with userId", async () => {
      const userCart = makeCart({ userId: "user-42", sessionId: null });
      mockClient.mutate.mockResolvedValueOnce({ createCart: userCart });

      const result = await cart.create({ userId: "user-42" });

      expect(result).toEqual(userCart);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.any(String),
        { input: { userId: "user-42" } },
      );
    });
  });

  describe("addItem", () => {
    it("should add an item with productId and quantity", async () => {
      const withItem = makeCart({
        items: [
          {
            id: "ci-1",
            productId: "prod-1",
            quantity: 2,
            unitPriceNanodollar: "5000000000",
          },
        ],
      });
      mockClient.mutate.mockResolvedValueOnce({ addCartItem: withItem });

      const result = await cart.addItem("cart-1", {
        productId: "prod-1",
        quantity: 2,
      });

      expect(result).toEqual(withItem);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("addCartItem"),
        {
          cartId: "cart-1",
          input: { productId: "prod-1", quantity: 2 },
        },
      );
    });

    it("should propagate GraphQLClientError when quantity <= 0", async () => {
      const err = new GraphQLClientError("GraphQL mutation failed", [
        { message: "quantity must be positive" },
      ]);
      mockClient.mutate.mockRejectedValueOnce(err);

      await expect(
        cart.addItem("cart-1", { productId: "prod-1", quantity: 0 }),
      ).rejects.toBeInstanceOf(GraphQLClientError);
    });
  });

  describe("updateItem", () => {
    it("should forward cartId, itemId, and quantity", async () => {
      const updated = makeCart({
        items: [
          {
            id: "ci-1",
            productId: "prod-1",
            quantity: 5,
            unitPriceNanodollar: "5000000000",
          },
        ],
      });
      mockClient.mutate.mockResolvedValueOnce({ updateCartItem: updated });

      const result = await cart.updateItem("cart-1", "ci-1", { quantity: 5 });

      expect(result).toEqual(updated);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("updateCartItem"),
        {
          cartId: "cart-1",
          itemId: "ci-1",
          input: { quantity: 5 },
        },
      );
    });
  });

  describe("removeItem", () => {
    it("should remove an item and return true", async () => {
      mockClient.mutate.mockResolvedValueOnce({ removeCartItem: true });

      const result = await cart.removeItem("cart-1", "ci-1");

      expect(result).toBe(true);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("removeCartItem"),
        { cartId: "cart-1", itemId: "ci-1" },
      );
    });
  });

  describe("clear", () => {
    it("should clear the cart and return true", async () => {
      mockClient.mutate.mockResolvedValueOnce({ clearCart: true });

      const result = await cart.clear("cart-1");

      expect(result).toBe(true);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("clearCart"),
        { cartId: "cart-1" },
      );
    });
  });

  describe("checkout", () => {
    it("should checkout pickup x in_store (TWS main flow)", async () => {
      const order = makeOrder({
        fulfillmentMethod: "pickup",
        paymentMethod: "in_store",
      });
      mockClient.mutate.mockResolvedValueOnce({ checkout: order });

      const result = await cart.checkout({
        cartId: "cart-1",
        fulfillmentMethod: "pickup",
        paymentMethod: "in_store",
      });

      expect(result).toEqual(order);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("checkout"),
        {
          input: {
            cartId: "cart-1",
            fulfillmentMethod: "pickup",
            paymentMethod: "in_store",
          },
        },
      );
    });

    it("should checkout delivery x online and return an order", async () => {
      const order = makeOrder({
        fulfillmentMethod: "delivery",
        paymentMethod: "online",
        shippingName: "Buyer",
        shippingAddress: "1-1-1 Shibuya",
        shippingPhone: "080-0000-0000",
      });
      mockClient.mutate.mockResolvedValueOnce({ checkout: order });

      const result = await cart.checkout({
        cartId: "cart-1",
        fulfillmentMethod: "delivery",
        paymentMethod: "online",
        shippingName: "Buyer",
        shippingAddress: "1-1-1 Shibuya",
        shippingPhone: "080-0000-0000",
      });

      expect(result).toEqual(order);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.any(String),
        {
          input: expect.objectContaining({
            cartId: "cart-1",
            fulfillmentMethod: "delivery",
            paymentMethod: "online",
          }),
        },
      );
    });

    it("should forward couponCode when supplied", async () => {
      const order = makeOrder({ discountNanodollar: "500000000" });
      mockClient.mutate.mockResolvedValueOnce({ checkout: order });

      await cart.checkout({
        cartId: "cart-1",
        fulfillmentMethod: "pickup",
        paymentMethod: "in_store",
        couponCode: "SAVE10",
      });

      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.any(String),
        { input: expect.objectContaining({ couponCode: "SAVE10" }) },
      );
    });

    it("should propagate GraphQLClientError for an invalid cartId", async () => {
      const err = new GraphQLClientError("GraphQL mutation failed", [
        { message: "Cart not found" },
      ]);
      mockClient.mutate.mockRejectedValueOnce(err);

      await expect(
        cart.checkout({
          cartId: "does-not-exist",
          fulfillmentMethod: "pickup",
          paymentMethod: "in_store",
        }),
      ).rejects.toBeInstanceOf(GraphQLClientError);
    });
  });
});
