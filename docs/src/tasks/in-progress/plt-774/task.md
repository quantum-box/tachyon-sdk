# PLT-774: storekit Phase 2a — 在庫管理SDK追加 (inventory API)

## 概要

`@tachyon-sdk/storekit` に inventory 操作を追加する。  
bakuure-api GraphQL スキーマの `productStock` / `adjustStock` を利用し、在庫取得・更新・低在庫一覧を提供する。

## 実装内容

- `packages/storekit/src/operations/inventory.ts` — `InventoryOperations` クラス
- `packages/storekit/src/__tests__/inventory.test.ts` — vitest ユニットテスト
- `packages/storekit/src/types.ts` — `StockInfo`, `ProductStock` 型追加
- `packages/storekit/src/index.ts` — 新型・クラスを export
- `packages/storekit/src/client.ts` — `inventory` プロパティ追加
- `packages/storekit/package.json` — version 0.1.0 → 0.2.0

## API

```typescript
inventory.getStock(productId: string): Promise<StockInfo>
inventory.updateStock(productId: string, quantity: number): Promise<StockInfo>
inventory.listLowStock(threshold?: number): Promise<ProductStock[]>
```

## スキーマ対応

| SDK メソッド | GraphQL | 備考 |
|---|---|---|
| `getStock` | `query productStock(productId)` | 直接対応 |
| `updateStock` | `mutation adjustStock(productId, input)` | `StockQuantityInput.quantity` を使用 |
| `listLowStock` | N/A | バックエンドに一覧クエリなし。scaffold実装 (TODO) |
