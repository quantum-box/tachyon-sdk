# @tachyon-sdk/storekit

Type-safe TypeScript SDK for bakuure commerce API.

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

## License

MIT
