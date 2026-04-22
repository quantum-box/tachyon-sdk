/**
 * GraphQL Client implementation using fetch API
 * Zero external dependencies
 */

import type {
  GraphQLRequestOptions,
  GraphQLResponse,
  GraphQLError,
} from "./types.js";

export class GraphQLClientError extends Error {
  public readonly errors: GraphQLError[];

  constructor(message: string, errors: GraphQLError[]) {
    super(message);
    this.name = "GraphQLClientError";
    this.errors = errors;
  }
}

export class GraphQLClient {
  private readonly url: string;
  private readonly options: GraphQLRequestOptions;

  constructor(url: string, options: GraphQLRequestOptions = {}) {
    this.url = url;
    this.options = options;
  }

  /**
   * Execute a GraphQL query
   */
  async query<T = unknown>(
    document: string,
    variables?: Record<string, unknown>,
  ): Promise<T> {
    return this.request<T>("query", document, variables);
  }

  /**
   * Execute a GraphQL mutation
   */
  async mutate<T = unknown>(
    document: string,
    variables?: Record<string, unknown>,
  ): Promise<T> {
    return this.request<T>("mutation", document, variables);
  }

  /**
   * Internal request handler
   */
  private async request<T>(
    operationType: "query" | "mutation",
    document: string,
    variables?: Record<string, unknown>,
  ): Promise<T> {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...this.options.headers,
    };

    if (this.options.apiKey) {
      headers["X-API-Key"] = this.options.apiKey;
    }

    if (this.options.bearerToken) {
      headers["Authorization"] = `Bearer ${this.options.bearerToken}`;
    }

    const response = await fetch(this.url, {
      method: "POST",
      headers,
      body: JSON.stringify({
        query: document,
        variables: variables ?? {},
      }),
    });

    const data = (await response.json()) as GraphQLResponse<T>;

    if (!response.ok) {
      throw new Error(
        `HTTP ${response.status}: ${response.statusText}${
          data.errors && data.errors.length > 0
            ? ` - ${data.errors[0].message}`
            : ""
        }`,
      );
    }

    if (data.errors && data.errors.length > 0) {
      throw new GraphQLClientError(
        `GraphQL ${operationType} failed`,
        data.errors,
      );
    }

    if (!data.data) {
      throw new Error(`No data returned from GraphQL ${operationType}`);
    }

    return data.data;
  }
}
