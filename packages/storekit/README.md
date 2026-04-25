# @tachyon-sdk/storekit

Type-safe TypeScript SDK for [bakuure](https://bakuure.app) commerce API.

Build storefronts on top of the bakuure multi-tenant commerce backend with a typed, ergonomic client.

## Features

- **Type-safe** — full TypeScript types for every request / response, generated against the live bakuure GraphQL schema
- **Namespaced API** — grouped entry points (`client.products`, `client.cart`, `client.orders`) keep call sites readable
- **Guest-friendly** — cart + checkout + order lookup work without user accounts; `sessionId` is enough
- **Structured errors** — `GraphQLClientError` exposes the GraphQL `errors[]` array for programmatic handling
- **Zero runtime dependencies** — the client is plain `fetch`, safe to use in Node.js, Cloudflare Workers, Deno, and browsers

## Installation

```bash
npm install @tachyon-sdk/storekit
```

## Quick Start

```typescript
import { StorekitClient } from "@tachyon-sdk/storekit";

// Initialize the client
const client = new StorekitClient({
  baseUrl: "https://api.example.com/graphql",
  apiKey: "sk_...", // or use bearerToken: "token"
});

// List products
const products = await client.products.list({ limit: 20 });

// Get a specific product
const product = await client.products.get("product-id");

// Create a cart
const cart = await client.cart.create({ sessionId: "session-123" });

// Add items to cart
await client.cart.addItem(cart.id, {
  productId: "product-id",
  quantity: 2,
});

// Update cart item
await client.cart.updateItem(cart.id, "item-id", { quantity: 3 });

// Remove item from cart
await client.cart.removeItem(cart.id, "item-id");

// Checkout
const order = await client.cart.checkout({
  cartId: cart.id,
  shippingName: "John Doe",
  shippingAddress: "123 Main St",
  shippingPhone: "+1-555-1234",
  fulfillmentMethod: "delivery",
  paymentMethod: "online",
});

// List orders
const orders = await client.orders.list({
  sessionId: "session-123",
  status: "confirmed",
  limit: 20,
});

// Get a specific order
const orderDetails = await client.orders.get(order.id);
```

## API Reference

### StorekitClient

Main client class that provides access to all API operations.

**Constructor Options:**

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `baseUrl` | `string` | Yes | GraphQL API endpoint URL |
| `apiKey` | `string` | No | API key for authentication |
| `bearerToken` | `string` | No | Bearer token for authentication |
| `headers` | `Record<string, string>` | No | Additional headers to include |

### Products

#### `client.products.list(input?: ProductsInput)`

Get a list of products.

**Input:**
```typescript
interface ProductsInput {
  limit?: number;    // default: 25
  offset?: number;  // default: 0
}
```

**Returns:** `ProductConnection`

#### `client.products.get(id: string)`

Get a single product by ID.

**Returns:** `Product`

### Cart

#### `client.cart.get(cartId: string)`

Get a cart by ID.

**Returns:** `Cart`

#### `client.cart.create(input?: CreateCartInput)`

Create a new cart.

**Input:**
```typescript
interface CreateCartInput {
  userId?: string;
  sessionId?: string;
}
```

**Returns:** `Cart`

#### `client.cart.addItem(cartId: string, input: AddCartItemInput)`

Add an item to the cart.

**Input:**
```typescript
interface AddCartItemInput {
  productId: string;
  quantity: number;
}
```

**Returns:** `Cart`

#### `client.cart.updateItem(cartId: string, itemId: string, input: UpdateCartItemInput)`

Update an item in the cart.

**Input:**
```typescript
interface UpdateCartItemInput {
  quantity: number;
}
```

**Returns:** `Cart`

#### `client.cart.removeItem(cartId: string, itemId: string)`

Remove an item from the cart.

**Returns:** `boolean`

#### `client.cart.clear(cartId: string)`

Clear all items from the cart.

**Returns:** `boolean`

#### `client.cart.checkout(input: CheckoutInput)`

Convert cart to order (checkout).

**Input:**
```typescript
interface CheckoutInput {
  cartId: string;
  shippingName?: string;
  shippingAddress?: string;
  shippingPhone?: string;
  fulfillmentMethod?: "pickup" | "delivery";
  paymentMethod?: "in_store" | "online";
  couponCode?: string;
}
```

**Returns:** `ConsumerOrder`

### Orders

#### `client.orders.list(input?: ConsumerOrdersInput)`

Get a list of consumer orders.

**Input:**
```typescript
interface ConsumerOrdersInput {
  userId?: string;
  sessionId?: string;
  status?: string;
  limit?: number;    // default: 20
  offset?: number;  // default: 0
}
```

**Returns:** `ConsumerOrderList`

#### `client.orders.get(orderId: string)`

Get a single order by ID.

**Returns:** `ConsumerOrder`

## Error Handling

The SDK throws errors for failed requests:

```typescript
import { GraphQLClientError } from "@tachyon-sdk/storekit";

try {
  const product = await client.products.get("invalid-id");
} catch (error) {
  if (error instanceof GraphQLClientError) {
    console.error("GraphQL errors:", error.errors);
  } else {
    console.error("Network or other error:", error);
  }
}
```

## Types

The SDK exports full TypeScript types for all API responses and inputs:

```typescript
import type {
  Product,
  Cart,
  CartItem,
  ConsumerOrder,
  CheckoutInput,
  // ... and more
} from "@tachyon-sdk/storekit";
```

## Development

```bash
# From the tachyon-sdk monorepo root
npm install

cd packages/storekit

npm run build         # tsc → dist/
npm test              # vitest run
npm run test:watch    # vitest (watch mode)
```

`prepublishOnly` runs `build` + `test`, so `npm publish` cannot ship without a green test suite and a fresh `dist/`.

## Contributing

Issues and pull requests are welcome. Before opening a PR:

1. Add or update tests (`vitest`) covering the change
2. Run `npm run build && npm test` locally
3. For non-trivial changes (new public methods, breaking changes), open a design issue first so API direction can be discussed

## Roadmap

Planned additions driven by production usage:

- **Guest-friendly order lookup** — fetch a guest order by phone number + last 4 digits of the order ID (no account required)
- **Cart reservation with TTL** — time-boxed stock hold on `cart.addItem` / `cart.updateItem` to prevent oversell
- **Out-of-stock availability states** — `availability: "in_stock" | "low_stock" | "out_of_stock"` on `Product`
- **Order notifications** — optional email receipt + printable receipt (HTML / PDF) for in-store pickup with QR-linked order lookup

## License

MIT — see [`LICENSE`](../../LICENSE) at the repository root.

---

Copyright © Quantum Box株式会社
