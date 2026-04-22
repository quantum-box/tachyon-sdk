/**
 * Unit tests for OrdersOperations
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { OrdersOperations } from "../operations/orders.js";
import { ConsumerOrderStatus } from "../types.js";
import type { ConsumerOrder } from "../types.js";

const makeOrder = (overrides: Partial<ConsumerOrder> = {}): ConsumerOrder => ({
  id: "order-1",
  tenantId: "tenant-1",
  cartId: "cart-1",
  userId: "user-1",
  sessionId: null,
  status: ConsumerOrderStatus.PENDING,
  fulfillmentMethod: "delivery",
  paymentMethod: "online",
  shippingName: "Test User",
  shippingAddress: "123 Main St",
  shippingPhone: "555-0100",
  subtotalNanodollar: "10000000000",
  discountNanodollar: "0",
  shippingFeeNanodollar: "500000000",
  totalNanodollar: "10500000000",
  items: [],
  confirmedAt: null,
  shippedAt: null,
  deliveredAt: null,
  createdAt: "2026-04-22T00:00:00Z",
  updatedAt: "2026-04-22T00:00:00Z",
  ...overrides,
});

describe("OrdersOperations", () => {
  let mockClient: { query: ReturnType<typeof vi.fn>; mutate: ReturnType<typeof vi.fn> };
  let orders: OrdersOperations;

  beforeEach(() => {
    mockClient = { query: vi.fn(), mutate: vi.fn() };
    orders = new OrdersOperations(mockClient as any);
  });

  describe("list", () => {
    it("should return a list of orders", async () => {
      const mockList = { items: [makeOrder()], limit: 20, offset: 0 };
      mockClient.query.mockResolvedValueOnce({ consumerOrders: mockList });

      const result = await orders.list({ userId: "user-1" });

      expect(result).toEqual(mockList);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("ConsumerOrders"),
        expect.objectContaining({ userId: "user-1" }),
      );
    });
  });

  describe("get", () => {
    it("should return a single order", async () => {
      const mockOrder = makeOrder();
      mockClient.query.mockResolvedValueOnce({ consumerOrder: mockOrder });

      const result = await orders.get("order-1");

      expect(result).toEqual(mockOrder);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("ConsumerOrder"),
        { orderId: "order-1" },
      );
    });
  });

  describe("updateStatus", () => {
    it("should call confirmOrder mutation for CONFIRMED status", async () => {
      const confirmed = makeOrder({ status: ConsumerOrderStatus.CONFIRMED });
      mockClient.mutate.mockResolvedValueOnce({ confirmOrder: confirmed });

      const result = await orders.updateStatus("order-1", ConsumerOrderStatus.CONFIRMED);

      expect(result).toEqual(confirmed);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("ConfirmOrder"),
        { orderId: "order-1" },
      );
    });

    it("should call shipOrder mutation for SHIPPED status", async () => {
      const shipped = makeOrder({ status: ConsumerOrderStatus.SHIPPED });
      mockClient.mutate.mockResolvedValueOnce({ shipOrder: shipped });

      const result = await orders.updateStatus("order-1", ConsumerOrderStatus.SHIPPED);

      expect(result).toEqual(shipped);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("ShipOrder"),
        { orderId: "order-1" },
      );
    });

    it("should call deliverOrder mutation for DELIVERED status", async () => {
      const delivered = makeOrder({ status: ConsumerOrderStatus.DELIVERED });
      mockClient.mutate.mockResolvedValueOnce({ deliverOrder: delivered });

      const result = await orders.updateStatus("order-1", ConsumerOrderStatus.DELIVERED);

      expect(result).toEqual(delivered);
    });

    it("should delegate to cancel() for CANCELLED status", async () => {
      const cancelled = makeOrder({ status: ConsumerOrderStatus.CANCELLED });
      mockClient.mutate.mockResolvedValueOnce({ cancelOrder: true });
      mockClient.query.mockResolvedValueOnce({ consumerOrder: cancelled });

      const result = await orders.updateStatus("order-1", ConsumerOrderStatus.CANCELLED);

      expect(result.status).toBe(ConsumerOrderStatus.CANCELLED);
    });

    it("should throw for unsupported status transitions", async () => {
      await expect(
        orders.updateStatus("order-1", ConsumerOrderStatus.PENDING),
      ).rejects.toThrow('no backend mutation for status "pending"');
    });
  });

  describe("cancel", () => {
    it("should call cancelOrder mutation then re-fetch the order", async () => {
      const cancelled = makeOrder({ status: ConsumerOrderStatus.CANCELLED });
      mockClient.mutate.mockResolvedValueOnce({ cancelOrder: true });
      mockClient.query.mockResolvedValueOnce({ consumerOrder: cancelled });

      const result = await orders.cancel("order-1", "customer request");

      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("CancelOrder"),
        { orderId: "order-1" },
      );
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("ConsumerOrder"),
        { orderId: "order-1" },
      );
      expect(result.status).toBe(ConsumerOrderStatus.CANCELLED);
    });

    it("should work without a reason argument", async () => {
      const cancelled = makeOrder({ status: ConsumerOrderStatus.CANCELLED });
      mockClient.mutate.mockResolvedValueOnce({ cancelOrder: true });
      mockClient.query.mockResolvedValueOnce({ consumerOrder: cancelled });

      await expect(orders.cancel("order-1")).resolves.toEqual(cancelled);
    });
  });

  describe("refund", () => {
    it("should throw not-implemented error (PLT-723 scaffold)", async () => {
      await expect(orders.refund("order-1", 1000)).rejects.toThrow(
        "Not implemented: requires PLT-723 approval",
      );
    });

    it("should throw without amount argument too", async () => {
      await expect(orders.refund("order-1")).rejects.toThrow("Not implemented");
    });
  });
});
