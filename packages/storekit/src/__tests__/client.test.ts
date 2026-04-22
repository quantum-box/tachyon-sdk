/**
 * Unit tests for GraphQLClient
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { GraphQLClient, GraphQLClientError } from "../graphql-client.js";

describe("GraphQLClient", () => {
  let client: GraphQLClient;
  let mockFetch: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockFetch = vi.fn();
    global.fetch = mockFetch as any;
    client = new GraphQLClient("https://api.example.com/graphql", {
      apiKey: "test-api-key",
    });
  });

  describe("query", () => {
    it("should send a query with proper headers", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          data: {
            products: {
              items: [],
              totalCount: 0,
              pageInfo: { limit: 25, offset: 0, hasNextPage: false },
            },
          },
        }),
      });

      const query = `query { products { items { id } } }`;
      await client.query(query);

      expect(mockFetch).toHaveBeenCalledWith(
        "https://api.example.com/graphql",
        expect.objectContaining({
          method: "POST",
          headers: expect.objectContaining({
            "Content-Type": "application/json",
            "X-API-Key": "test-api-key",
          }),
          body: expect.stringContaining(query),
        }),
      );
    });

    it("should return data on successful query", async () => {
      const mockData = {
        products: {
          items: [
            {
              id: "prod-1",
              name: "Test Product",
              listPrice: 1000,
            },
          ],
          totalCount: 1,
          pageInfo: { limit: 25, offset: 0, hasNextPage: false },
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockData }),
      });

      const result = await client.query(mockData);

      expect(result).toEqual(mockData);
    });

    it("should throw on HTTP error", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
        json: async () => ({ errors: [] }),
      });

      await expect(
        client.query(`query { products { items { id } } }`),
      ).rejects.toThrow("HTTP 500: Internal Server Error");
    });

    it("should throw GraphQLClientError on GraphQL errors", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          errors: [
            {
              message: "Validation error",
              locations: [{ line: 2, column: 3 }],
              path: ["products"],
            },
          ],
        }),
      });

      await expect(
        client.query(`query { products { items { id } } }`),
      ).rejects.toThrow(GraphQLClientError);
    });

    it("should throw on missing data", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      });

      await expect(
        client.query(`query { products { items { id } } }`),
      ).rejects.toThrow("No data returned");
    });
  });

  describe("mutate", () => {
    it("should send a mutation", async () => {
      const mockData = {
        createCart: {
          id: "cart-1",
          items: [],
          status: "active",
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockData }),
      });

      const mutation = `mutation CreateCart($input: CreateCartInput!) { createCart(input: $input) { id } }`;
      const result = await client.mutate(mutation, { input: {} });

      expect(result).toEqual(mockData);
    });

    it("should throw on mutation errors", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          errors: [{ message: "Cart creation failed" }],
        }),
      });

      await expect(
        client.mutate(`mutation CreateCart { createCart { id } }`),
      ).rejects.toThrow(GraphQLClientError);
    });
  });

  describe("authentication", () => {
    it("should use API key from config", async () => {
      client = new GraphQLClient("https://api.example.com/graphql", {
        apiKey: "my-secret-key",
      });

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: { products: {} } }),
      });

      await client.query(`query { products { items { id } } }`);

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            "X-API-Key": "my-secret-key",
          }),
        }),
      );
    });

    it("should use bearer token from config", async () => {
      client = new GraphQLClient("https://api.example.com/graphql", {
        bearerToken: "my-token",
      });

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: { products: {} } }),
      });

      await client.query(`query { products { items { id } } }`);

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            "Authorization": "Bearer my-token",
          }),
        }),
      );
    });

    it("should merge custom headers", async () => {
      client = new GraphQLClient("https://api.example.com/graphql", {
        apiKey: "test-key",
        headers: {
          "X-Custom-Header": "custom-value",
        },
      });

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: { products: {} } }),
      });

      await client.query(`query { products { items { id } } }`);

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            "X-API-Key": "test-key",
            "X-Custom-Header": "custom-value",
          }),
        }),
      );
    });
  });
});
