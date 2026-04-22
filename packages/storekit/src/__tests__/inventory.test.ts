/**
 * Unit tests for InventoryOperations
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { InventoryOperations } from "../operations/inventory.js";

const mockStockInfo = {
  id: "stock-1",
  productId: "prod-1",
  quantityOnHand: 100,
  quantityReserved: 10,
  quantityAvailable: 90,
  lowStockThreshold: 5,
  trackInventory: true,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-02T00:00:00Z",
};

describe("InventoryOperations", () => {
  let inventory: InventoryOperations;
  let mockClient: { query: ReturnType<typeof vi.fn>; mutate: ReturnType<typeof vi.fn> };

  beforeEach(() => {
    mockClient = { query: vi.fn(), mutate: vi.fn() };
    inventory = new InventoryOperations(mockClient as any);
  });

  describe("getStock", () => {
    it("should return stock info for a product", async () => {
      mockClient.query.mockResolvedValueOnce({ productStock: mockStockInfo });

      const result = await inventory.getStock("prod-1");

      expect(result).toEqual(mockStockInfo);
      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining("productStock"),
        { productId: "prod-1" },
      );
    });

    it("should pass the productId variable", async () => {
      mockClient.query.mockResolvedValueOnce({ productStock: { ...mockStockInfo, productId: "prod-99" } });

      await inventory.getStock("prod-99");

      expect(mockClient.query).toHaveBeenCalledWith(
        expect.any(String),
        { productId: "prod-99" },
      );
    });
  });

  describe("updateStock", () => {
    it("should adjust stock and return updated info", async () => {
      const updated = { ...mockStockInfo, quantityOnHand: 120, quantityAvailable: 110 };
      mockClient.mutate.mockResolvedValueOnce({ adjustStock: updated });

      const result = await inventory.updateStock("prod-1", 20);

      expect(result).toEqual(updated);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining("adjustStock"),
        { productId: "prod-1", input: { quantity: 20 } },
      );
    });

    it("should support negative quantity for stock reduction", async () => {
      const updated = { ...mockStockInfo, quantityOnHand: 80, quantityAvailable: 70 };
      mockClient.mutate.mockResolvedValueOnce({ adjustStock: updated });

      const result = await inventory.updateStock("prod-1", -20);

      expect(result).toEqual(updated);
      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.any(String),
        { productId: "prod-1", input: { quantity: -20 } },
      );
    });
  });

  describe("listLowStock", () => {
    it("should throw because the API does not support this query yet", async () => {
      await expect(inventory.listLowStock()).rejects.toThrow(
        "listLowStock is not yet supported",
      );
    });

    it("should throw even when a threshold is provided", async () => {
      await expect(inventory.listLowStock(5)).rejects.toThrow(
        "listLowStock is not yet supported",
      );
    });
  });
});
