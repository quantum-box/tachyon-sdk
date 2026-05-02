# StoreKit

`StoreKit` is the storefront contract for external EC sites. The primary SDK source is `bakuure-api/bakuure.openapi.yaml` and its `/v1/storekit/*` REST endpoints. This package also includes a small ergonomic TypeScript wrapper and a GraphQL adapter for Bakuure UI/developer workflows that already use `bakuure-api/schema.graphql`.

## Create a Client

### OpenAPI REST client

Prefer an OpenAPI-generated client from `bakuure-api/bakuure.openapi.yaml` for external storefront SDKs. The hand-authored wrapper below follows the same `/v1/storekit/*` contract.

```ts
import { createStoreKitClient } from '@tachyon/sdk'

const storekit = createStoreKitClient({
  baseUrl: 'https://bakuure.api.n1.tachy.one',
  operatorId: 'tn_...',
  publicApiKey: '...',
  sessionId: 'guest-session-id',
})
```

### Generated GraphQL SDK adapter

Use this path when building UI code against `bakuure-api/schema.graphql` and the generated GraphQL SDK.

```ts
import { GraphQLClient } from 'graphql-request'
import { getSdk } from './gen/graphql'
import { createStoreKitClientFromGraphqlSdk } from '@tachyon/sdk'

const graphql = getSdk(
  new GraphQLClient('https://bakuure.api.n1.tachy.one/v1/graphql', {
    headers: { 'x-operator-id': 'tn_...' },
  }),
)

const storekit = createStoreKitClientFromGraphqlSdk({
  sdk: graphql,
  sessionId: 'guest-session-id',
})
```

## Basic Flow

```ts
const products = await storekit.products.list({ limit: 20 })
const product = await storekit.products.get(products.data[0].id)

const cart = await storekit.cart.create()
const updatedCart = await storekit.cart.addItem({
  cartId: cart.id,
  productId: product.id,
  quantity: 1,
})

const coupon = await storekit.coupons.validate({
  code: 'WELCOME',
  subtotalNanodollar: updatedCart.items[0].unitPrice.nanodollar,
})

const checkout = await storekit.checkout.create({
  cartId: updatedCart.id,
  couponCode: coupon.code,
  successUrl: 'https://example.com/thanks',
  cancelUrl: 'https://example.com/cart',
})
```

## Resources

- `products.list`, `products.get`, `products.listCategories`, `products.getStock`
- `cart.create`, `cart.get`, `cart.addItem`, `cart.updateItem`, `cart.removeItem`, `cart.clear`
- `coupons.validate`
- `checkout.create`, `checkout.confirm`
- `orders.list`, `orders.get`, `orders.cancel`, `orders.selectPickupDatetime`
- `orders.prepare`, `orders.ship`, `orders.deliver`, `orders.ready`, `orders.pickup`, `orders.refund`
- `customers.create`, `customers.list`, `customers.get`, `customers.update`, `customers.delete`

Cart updates and deletes are item-id based. Use the `id` returned in `cart.items[]`, not the product ID.

The GraphQL adapter currently covers the consumer shop flow generated from `bakuure-api/schema.graphql`: products, product detail with stock, cart, coupons, checkout, pickup datetime selection, order list, and order detail. Customer management and merchant-only order transitions are covered by the REST/OpenAPI StoreKit contract.

## Errors

All non-2xx responses are normalized to `StoreKitError`.

```ts
try {
  await storekit.products.get('missing')
} catch (error) {
  if (error instanceof StoreKitError) {
    console.log(error.status, error.type, error.code, error.param)
  }
}
```
