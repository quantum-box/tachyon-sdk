import { describe, expect, it, vi } from "vitest";
import {
  StoreKitPaymentProviderRegistry,
  StoreKitPaymentProviderSelectionError,
  StoreKitPaymentStateTransitionError,
  transitionStoreKitPaymentStatus,
} from "../payment-registry.js";
import type {
  StoreKitPayment,
  StoreKitPaymentEvent,
  StoreKitPaymentGetInput,
  StoreKitPaymentProvider,
  StoreKitPaymentProviderAdapter,
  StoreKitRefundInput,
  StoreKitPaymentCancelInput,
  StoreKitPaymentCreateInput,
  StoreKitVerifiedWebhook,
  StoreKitWebhookVerifyInput,
} from "../types.js";

const makePayment = (
  provider: StoreKitPaymentProvider,
  overrides: Partial<StoreKitPayment> = {},
): StoreKitPayment => ({
  id: `payment-${provider}`,
  tenantId: "tenant-1",
  provider,
  kind: "invoice_payment",
  status: "pending",
  amountMinor: 1000,
  currency: "USD",
  providerPaymentId: `${provider}-payment-1`,
  idempotencyKey: "idem-1",
  createdAt: "2026-06-12T00:00:00.000Z",
  updatedAt: "2026-06-12T00:00:00.000Z",
  ...overrides,
});

const makeAdapter = (provider: StoreKitPaymentProvider): StoreKitPaymentProviderAdapter => {
  const payment = makePayment(provider);

  return {
    provider,
    createPayment: vi.fn(async (_input: StoreKitPaymentCreateInput) => payment),
    getPayment: vi.fn(async (_input: StoreKitPaymentGetInput) => payment),
    cancelPayment: vi.fn(async (_input: StoreKitPaymentCancelInput) => ({
      ...payment,
      status: "cancelled",
    })),
    refundPayment: vi.fn(async (_input: StoreKitRefundInput) => ({
      ...payment,
      status: "refunded",
    })),
    verifyWebhook: vi.fn(async (_input: StoreKitWebhookVerifyInput): Promise<StoreKitVerifiedWebhook> => ({
      provider,
      tenantId: payment.tenantId,
      eventId: "event-1",
      rawEventType: "provider.event",
      receivedAt: "2026-06-12T00:00:00.000Z",
      payment,
    })),
    normalizeWebhook: vi.fn(async (input: StoreKitVerifiedWebhook): Promise<StoreKitPaymentEvent> => ({
      provider,
      eventId: input.eventId,
      eventType: "payment_unknown",
      payment: input.payment,
      receivedAt: input.receivedAt,
      rawEventType: input.rawEventType,
    })),
  };
};

describe("StoreKitPaymentProviderRegistry", () => {
  it("selects providers by explicit priority rather than registration order", () => {
    const stripe = makeAdapter("stripe");
    const square = makeAdapter("square");
    const registry = new StoreKitPaymentProviderRegistry([
      { adapter: stripe },
      { adapter: square },
    ]);

    expect(registry.select(["square", "stripe"])).toBe(square);
    expect(registry.select(["stripe", "square"])).toBe(stripe);
  });

  it("skips unavailable providers during registration and warns without registering them", () => {
    const warn = vi.fn();
    const square = makeAdapter("square");
    const stripe = makeAdapter("stripe");
    const registry = new StoreKitPaymentProviderRegistry(
      [
        { adapter: stripe, isConfigured: false, unavailableReason: "missing webhook secret" },
        { adapter: square },
      ],
      { warn },
    );

    expect(registry.listRegisteredProviders()).toEqual(["square"]);
    expect(registry.select(["stripe", "square"])).toBe(square);
    expect(warn).toHaveBeenCalledWith(
      "Skipping unavailable StoreKit payment provider",
      { provider: "stripe", reason: "missing webhook secret" },
    );
  });

  it("throws a typed error when priority contains no usable provider", () => {
    const registry = new StoreKitPaymentProviderRegistry([
      { adapter: makeAdapter("square"), isConfigured: false },
    ]);

    expect(() => registry.select(["square"])).toThrow(StoreKitPaymentProviderSelectionError);
  });
});

describe("transitionStoreKitPaymentStatus", () => {
  it("documents and applies supported success, cancellation, refund, and expiration transitions", () => {
    expect(transitionStoreKitPaymentStatus("pending", "payment_paid")).toBe("paid");
    expect(transitionStoreKitPaymentStatus("requires_action", "payment_failed")).toBe("failed");
    expect(transitionStoreKitPaymentStatus("pending", "payment_expired")).toBe("cancelled");
    expect(transitionStoreKitPaymentStatus("paid", "payment_refunded")).toBe("refunded");
  });

  it("keeps partial refunds, dispute events, and unknown events from blindly overwriting payment status", () => {
    expect(transitionStoreKitPaymentStatus("paid", "payment_partially_refunded")).toBe("paid");
    expect(transitionStoreKitPaymentStatus("paid", "payment_dispute_opened")).toBe("paid");
    expect(transitionStoreKitPaymentStatus("paid", "payment_dispute_closed")).toBe("paid");
    expect(
      transitionStoreKitPaymentStatus("paid", "payment_dispute_closed", {
        disputeResult: "lost_or_refunded",
      }),
    ).toBe("refunded");
    expect(transitionStoreKitPaymentStatus("pending", "payment_unknown")).toBe("pending");
  });

  it("rejects undefined transitions and terminal-state mutations", () => {
    expect(() => transitionStoreKitPaymentStatus("paid", "cancelPayment")).toThrow(
      StoreKitPaymentStateTransitionError,
    );
    expect(() => transitionStoreKitPaymentStatus("cancelled", "payment_paid")).toThrow(
      StoreKitPaymentStateTransitionError,
    );
  });
});
