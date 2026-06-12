import { createHmac } from "node:crypto";
import { describe, expect, it, vi } from "vitest";
import {
  StoreKitSquareAdapter,
  StoreKitSquareTenantMismatchError,
  StoreKitSquareWebhookSignatureError,
  verifySquareSignature,
} from "../adapters/square.js";
import type {
  StoreKitPayment,
  StoreKitSquareClient,
  StoreKitSquarePaymentStore,
} from "../index.js";

class FakeSquarePaymentStore implements StoreKitSquarePaymentStore {
  private readonly payments = new Map<string, StoreKitPayment>();

  async save(payment: StoreKitPayment): Promise<StoreKitPayment> {
    this.payments.set(payment.id, payment);
    return payment;
  }

  async getById(paymentId: string): Promise<StoreKitPayment | undefined> {
    return this.payments.get(paymentId);
  }

  async getByProviderPaymentId(providerPaymentId: string): Promise<StoreKitPayment | undefined> {
    return [...this.payments.values()].find((payment) => payment.providerPaymentId === providerPaymentId);
  }

  async getByProviderReference(providerReference: string): Promise<StoreKitPayment | undefined> {
    return [...this.payments.values()].find((payment) => payment.providerReference === providerReference);
  }
}

const notificationUrl = "https://example.test/webhooks/square";
const signatureKey = "test-square-webhook-signature-key";
const fixedDate = new Date("2026-06-12T00:00:00.000Z");

function sign(rawBody: string): string {
  return createHmac("sha256", signatureKey)
    .update(`${notificationUrl}${rawBody}`)
    .digest("base64");
}

function makeAdapter(overrides: Partial<StoreKitSquareClient> = {}) {
  const store = new FakeSquarePaymentStore();
  const client: StoreKitSquareClient = {
    createPaymentLink: vi.fn(async () => ({
      id: "plink-1",
      url: "https://square.example/checkout/plink-1",
      order_id: "order-1",
      created_at: fixedDate.toISOString(),
      updated_at: fixedDate.toISOString(),
    })),
    getPayment: vi.fn(async () => undefined),
    cancelPayment: vi.fn(async () => ({
      id: "payment-1",
      status: "CANCELED",
      updated_at: fixedDate.toISOString(),
    })),
    refundPayment: vi.fn(async () => ({
      id: "refund-1",
      status: "COMPLETED",
      amount_money: { amount: 1000, currency: "USD" },
      payment_id: "payment-1",
      updated_at: fixedDate.toISOString(),
    })),
    ...overrides,
  };

  const adapter = new StoreKitSquareAdapter({
    client,
    store,
    webhookSignatureKey: signatureKey,
    webhookNotificationUrl: notificationUrl,
    now: () => fixedDate,
    idFactory: () => "storekit-payment-1",
  });

  return { adapter, client, store };
}

async function createStoredPayment(adapter: StoreKitSquareAdapter): Promise<StoreKitPayment> {
  return adapter.createPayment({
    tenantId: "tenant-1",
    kind: "invoice_payment",
    amountMinor: 1000,
    currency: "USD",
    reference: { invoiceId: "invoice-1" },
    customer: { email: "customer@example.test" },
    successUrl: "https://example.test/success",
    cancelUrl: "https://example.test/cancel",
    idempotencyKey: "idem-create-1",
  });
}

describe("StoreKitSquareAdapter", () => {
  it("creates a StoreKit payment through an injected Square client without network credentials", async () => {
    const { adapter, client } = makeAdapter();

    const payment = await createStoredPayment(adapter);

    expect(payment).toMatchObject({
      id: "storekit-payment-1",
      tenantId: "tenant-1",
      provider: "square",
      status: "pending",
      amountMinor: 1000,
      currency: "USD",
      checkoutUrl: "https://square.example/checkout/plink-1",
      providerPaymentId: "plink-1",
      providerReference: "order-1",
      idempotencyKey: "idem-create-1",
    });
    expect(client.createPaymentLink).toHaveBeenCalledWith({
      idempotencyKey: "idem-create-1",
      amountMinor: 1000,
      currency: "USD",
      reference: { invoiceId: "invoice-1" },
      successUrl: "https://example.test/success",
      cancelUrl: "https://example.test/cancel",
      customer: { email: "customer@example.test" },
    });
  });

  it("verifies Square webhook signatures with a constant-time HMAC comparison input", () => {
    const rawBody = JSON.stringify({ event_id: "event-1", type: "payment.updated" });

    expect(verifySquareSignature({
      signature: sign(rawBody),
      rawBody,
      signatureKey,
      notificationUrl,
    })).toBe(true);
    expect(verifySquareSignature({
      signature: sign(rawBody).replace(/.$/, "A"),
      rawBody,
      signatureKey,
      notificationUrl,
    })).toBe(false);
  });

  it("normalizes completed Square payment webhooks to paid and preserves tenant checks", async () => {
    const { adapter } = makeAdapter();
    await createStoredPayment(adapter);
    const rawBody = JSON.stringify({
      event_id: "event-paid-1",
      type: "payment.updated",
      created_at: "2026-06-12T00:01:00.000Z",
      data: {
        id: "payment-1",
        type: "payment",
        object: {
          payment: {
            id: "payment-1",
            status: "COMPLETED",
            amount_money: { amount: 1000, currency: "USD" },
            order_id: "order-1",
            updated_at: "2026-06-12T00:01:00.000Z",
          },
        },
      },
    });

    const verified = await adapter.verifyWebhook({
      tenantId: "tenant-1",
      rawBody,
      headers: { "x-square-hmacsha256-signature": sign(rawBody) },
    });
    const event = await adapter.normalizeWebhook(verified);

    expect(event).toMatchObject({
      provider: "square",
      eventId: "event-paid-1",
      eventType: "payment_paid",
      rawEventType: "payment.updated",
      receivedAt: "2026-06-12T00:01:00.000Z",
    });
    expect(event.payment).toMatchObject({
      id: "storekit-payment-1",
      tenantId: "tenant-1",
      status: "paid",
      providerPaymentId: "payment-1",
      providerReference: "order-1",
    });
  });

  it("rejects invalid signatures and tenant mismatches before normalization", async () => {
    const { adapter } = makeAdapter();
    await createStoredPayment(adapter);
    const rawBody = JSON.stringify({
      event_id: "event-paid-1",
      type: "payment.updated",
      data: {
        object: {
          payment: {
            id: "payment-1",
            status: "COMPLETED",
            order_id: "order-1",
          },
        },
      },
    });

    await expect(adapter.verifyWebhook({
      tenantId: "tenant-1",
      rawBody,
      headers: { "x-square-hmacsha256-signature": "bad" },
    })).rejects.toThrow(StoreKitSquareWebhookSignatureError);

    await expect(adapter.verifyWebhook({
      tenantId: "tenant-2",
      rawBody,
      headers: { "x-square-hmacsha256-signature": sign(rawBody) },
    })).rejects.toThrow(StoreKitSquareTenantMismatchError);
  });

  it("normalizes cancellation, failed, expired, refund, partial refund, dispute, and unknown events safely", async () => {
    const { adapter } = makeAdapter();
    const payment = await createStoredPayment(adapter);

    const cases = [
      {
        rawEventType: "payment.updated",
        object: { payment: { id: "payment-1", status: "CANCELED", order_id: "order-1" } },
        expected: "payment_cancelled",
      },
      {
        rawEventType: "payment.updated",
        object: { payment: { id: "payment-1", status: "FAILED", order_id: "order-1" } },
        expected: "payment_failed",
      },
      {
        rawEventType: "payment_link.expired",
        object: { payment_link: { id: "plink-1", order_id: "order-1" } },
        expected: "payment_expired",
      },
      {
        rawEventType: "refund.updated",
        object: { refund: { id: "refund-1", status: "COMPLETED", payment_id: "payment-1", amount_money: { amount: 1000, currency: "USD" } } },
        expected: "payment_refunded",
      },
      {
        rawEventType: "refund.updated",
        object: { refund: { id: "refund-2", status: "COMPLETED", payment_id: "payment-1", amount_money: { amount: 250, currency: "USD" } } },
        expected: "payment_partially_refunded",
      },
      {
        rawEventType: "dispute.created",
        object: { dispute: { id: "dispute-1", state: "INQUIRY", payment_id: "payment-1" } },
        expected: "payment_dispute_opened",
      },
      {
        rawEventType: "dispute.updated",
        object: { dispute: { id: "dispute-1", state: "LOST", payment_id: "payment-1" } },
        expected: "payment_dispute_closed",
      },
      {
        rawEventType: "customer.updated",
        object: {},
        expected: "payment_unknown",
      },
    ] as const;

    for (const [index, testCase] of cases.entries()) {
      const event = await adapter.normalizeWebhook({
        provider: "square",
        tenantId: payment.tenantId,
        eventId: `event-${index}`,
        rawEventType: testCase.rawEventType,
        receivedAt: fixedDate.toISOString(),
        payment: testCase.expected === "payment_unknown" ? undefined : payment,
        rawEvent: {
          event_id: `event-${index}`,
          type: testCase.rawEventType,
          data: { object: testCase.object },
        },
      });

      expect(event.eventType).toBe(testCase.expected);
      if (testCase.expected === "payment_unknown") {
        expect(event.payment).toBeUndefined();
      }
    }
  });

  it("uses idempotency keys for cancel and refund operations", async () => {
    const { adapter, client } = makeAdapter();
    const payment = await createStoredPayment(adapter);

    await adapter.cancelPayment({
      tenantId: "tenant-1",
      paymentId: payment.id,
      idempotencyKey: "idem-cancel-1",
    });
    await adapter.refundPayment({
      tenantId: "tenant-1",
      paymentId: payment.id,
      amountMinor: 1000,
      reason: "requested_by_customer",
      idempotencyKey: "idem-refund-1",
    });

    expect(client.cancelPayment).toHaveBeenCalledWith("plink-1", "idem-cancel-1");
    expect(client.refundPayment).toHaveBeenCalledWith({
      paymentId: "plink-1",
      amountMinor: 1000,
      reason: "requested_by_customer",
      idempotencyKey: "idem-refund-1",
    });
  });
});
