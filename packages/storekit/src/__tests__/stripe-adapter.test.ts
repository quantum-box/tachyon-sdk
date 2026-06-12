import { createHmac } from "node:crypto";
import { describe, expect, it, vi } from "vitest";
import {
  StoreKitStripeAdapter,
  StoreKitStripeTenantMismatchError,
  StoreKitStripeWebhookSignatureError,
  verifyStripeSignature,
} from "../adapters/stripe.js";
import type {
  StoreKitPayment,
  StoreKitStripeClient,
  StoreKitStripePaymentStore,
} from "../index.js";

class FakeStripePaymentStore implements StoreKitStripePaymentStore {
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

const webhookSecret = "test-stripe-webhook-signature-secret";
const fixedDate = new Date("2026-06-12T00:00:00.000Z");
const fixedTimestamp = Math.floor(fixedDate.getTime() / 1000);

function sign(rawBody: string, timestamp = fixedTimestamp): string {
  const signature = createHmac("sha256", webhookSecret)
    .update(`${timestamp}.${rawBody}`)
    .digest("hex");
  return `t=${timestamp},v1=${signature}`;
}

function makeAdapter(overrides: Partial<StoreKitStripeClient> = {}) {
  const store = new FakeStripePaymentStore();
  const client: StoreKitStripeClient = {
    createCheckoutSession: vi.fn(async () => ({
      id: "cs_test_1",
      url: "https://checkout.stripe.example/cs_test_1",
      payment_intent: "pi_test_1",
      payment_status: "unpaid",
      status: "open",
      amount_total: 1000,
      currency: "usd",
      created: fixedTimestamp,
    })),
    getPaymentIntent: vi.fn(async () => undefined),
    cancelPaymentIntent: vi.fn(async () => ({
      id: "pi_test_1",
      status: "canceled",
      canceled_at: fixedTimestamp,
    })),
    refundPayment: vi.fn(async () => ({
      id: "re_test_1",
      status: "succeeded",
      payment_intent: "pi_test_1",
      amount: 1000,
      currency: "usd",
      created: fixedTimestamp,
    })),
    ...overrides,
  };

  const adapter = new StoreKitStripeAdapter({
    client,
    store,
    webhookSecret,
    now: () => fixedDate,
    idFactory: () => "storekit-payment-1",
  });

  return { adapter, client, store };
}

async function createStoredPayment(adapter: StoreKitStripeAdapter): Promise<StoreKitPayment> {
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

describe("StoreKitStripeAdapter", () => {
  it("creates a StoreKit payment through an injected Stripe client without network credentials", async () => {
    const { adapter, client } = makeAdapter();

    const payment = await createStoredPayment(adapter);

    expect(payment).toMatchObject({
      id: "storekit-payment-1",
      tenantId: "tenant-1",
      provider: "stripe",
      status: "pending",
      amountMinor: 1000,
      currency: "USD",
      checkoutUrl: "https://checkout.stripe.example/cs_test_1",
      providerPaymentId: "pi_test_1",
      providerReference: "cs_test_1",
      idempotencyKey: "idem-create-1",
    });
    expect(client.createCheckoutSession).toHaveBeenCalledWith({
      idempotencyKey: "idem-create-1",
      amountMinor: 1000,
      currency: "USD",
      reference: { invoiceId: "invoice-1" },
      successUrl: "https://example.test/success",
      cancelUrl: "https://example.test/cancel",
      customer: { email: "customer@example.test" },
      metadata: {
        storekitPaymentId: "storekit-payment-1",
        storekitTenantId: "tenant-1",
        storekitKind: "invoice_payment",
        invoiceId: "invoice-1",
        reservationId: undefined,
        fieldPaymentIntentId: undefined,
        orderId: undefined,
      },
    });
  });

  it("verifies Stripe webhook signatures with timestamp tolerance and HMAC comparison", () => {
    const rawBody = JSON.stringify({ id: "evt_1", type: "payment_intent.succeeded" });

    expect(verifyStripeSignature({
      signature: sign(rawBody),
      rawBody,
      webhookSecret,
      now: () => fixedDate,
    })).toBe(true);
    expect(verifyStripeSignature({
      signature: `t=${fixedTimestamp},v1=${"0".repeat(64)}`,
      rawBody,
      webhookSecret,
      now: () => fixedDate,
    })).toBe(false);
    expect(verifyStripeSignature({
      signature: sign(rawBody, fixedTimestamp - 1000),
      rawBody,
      webhookSecret,
      now: () => fixedDate,
    })).toBe(false);
  });

  it("normalizes successful PaymentIntent webhooks to paid and preserves tenant checks", async () => {
    const { adapter } = makeAdapter();
    await createStoredPayment(adapter);
    const rawBody = JSON.stringify({
      id: "evt_paid_1",
      type: "payment_intent.succeeded",
      created: fixedTimestamp + 60,
      data: {
        object: {
          id: "pi_test_1",
          status: "succeeded",
          amount: 1000,
          currency: "usd",
          metadata: { storekitPaymentId: "storekit-payment-1" },
        },
      },
    });

    const verified = await adapter.verifyWebhook({
      tenantId: "tenant-1",
      rawBody,
      headers: { "stripe-signature": sign(rawBody, fixedTimestamp + 60) },
    });
    const event = await adapter.normalizeWebhook(verified);

    expect(event).toMatchObject({
      provider: "stripe",
      eventId: "evt_paid_1",
      eventType: "payment_paid",
      rawEventType: "payment_intent.succeeded",
      receivedAt: "2026-06-12T00:01:00.000Z",
    });
    expect(event.payment).toMatchObject({
      id: "storekit-payment-1",
      tenantId: "tenant-1",
      status: "paid",
      providerPaymentId: "pi_test_1",
    });
  });

  it("rejects invalid signatures and tenant mismatches before normalization", async () => {
    const { adapter } = makeAdapter();
    await createStoredPayment(adapter);
    const rawBody = JSON.stringify({
      id: "evt_paid_1",
      type: "payment_intent.succeeded",
      data: {
        object: {
          id: "pi_test_1",
          status: "succeeded",
          metadata: { storekitPaymentId: "storekit-payment-1" },
        },
      },
    });

    await expect(adapter.verifyWebhook({
      tenantId: "tenant-1",
      rawBody,
      headers: { "stripe-signature": "bad" },
    })).rejects.toThrow(StoreKitStripeWebhookSignatureError);

    await expect(adapter.verifyWebhook({
      tenantId: "tenant-2",
      rawBody,
      headers: { "stripe-signature": sign(rawBody) },
    })).rejects.toThrow(StoreKitStripeTenantMismatchError);
  });

  it("normalizes cancellation, failed, expired, refund, partial refund, dispute, and unknown events safely", async () => {
    const { adapter } = makeAdapter();
    const payment = await createStoredPayment(adapter);

    const cases = [
      {
        rawEventType: "payment_intent.canceled",
        object: { id: "pi_test_1", status: "canceled", metadata: { storekitPaymentId: payment.id } },
        expected: "payment_cancelled",
      },
      {
        rawEventType: "payment_intent.payment_failed",
        object: { id: "pi_test_1", status: "requires_payment_method", metadata: { storekitPaymentId: payment.id } },
        expected: "payment_failed",
      },
      {
        rawEventType: "checkout.session.expired",
        object: { id: "cs_test_1", status: "expired", payment_status: "unpaid", payment_intent: "pi_test_1" },
        expected: "payment_expired",
      },
      {
        rawEventType: "refund.updated",
        object: { id: "re_test_1", status: "succeeded", payment_intent: "pi_test_1", amount: 1000, currency: "usd" },
        expected: "payment_refunded",
      },
      {
        rawEventType: "refund.updated",
        object: { id: "re_test_2", status: "succeeded", payment_intent: "pi_test_1", amount: 250, currency: "usd" },
        expected: "payment_partially_refunded",
      },
      {
        rawEventType: "charge.dispute.created",
        object: { id: "dp_test_1", status: "needs_response", payment_intent: "pi_test_1" },
        expected: "payment_dispute_opened",
      },
      {
        rawEventType: "charge.dispute.closed",
        object: { id: "dp_test_1", status: "lost", payment_intent: "pi_test_1" },
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
        provider: "stripe",
        tenantId: payment.tenantId,
        eventId: `evt_${index}`,
        rawEventType: testCase.rawEventType,
        receivedAt: fixedDate.toISOString(),
        payment: testCase.expected === "payment_unknown" ? undefined : payment,
        rawEvent: {
          id: `evt_${index}`,
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

    expect(client.cancelPaymentIntent).toHaveBeenCalledWith("pi_test_1", "idem-cancel-1");
    expect(client.refundPayment).toHaveBeenCalledWith({
      paymentIntentId: "pi_test_1",
      amountMinor: 1000,
      reason: "requested_by_customer",
      idempotencyKey: "idem-refund-1",
    });
  });
});
