# PLT-775: storekit Phase 2b — 注文管理SDK拡充 (更新/キャンセル/返金)

## 概要

`packages/storekit` の注文操作を拡充し、ステータス更新・キャンセル・返金 (scaffold) を追加する。

## 実装 API

| メソッド | 説明 | 状態 |
|---|---|---|
| `orders.updateStatus(orderId, status)` | ステータス遷移 mutation に委譲 | 実装 |
| `orders.cancel(orderId, reason?)` | `cancelOrder` mutation → re-fetch | 実装 |
| `orders.refund(orderId, amount?)` | scaffold — PLT-723 判断待ち | scaffold |

## 依存

- PLT-723: 返金処理 CEO 判断 (refund は scaffold のみ)
- PLT-578: storekit 初版 (list/get 実装済み)
