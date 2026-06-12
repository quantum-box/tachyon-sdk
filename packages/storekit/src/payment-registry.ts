import type {
  StoreKitPaymentEventType,
  StoreKitPaymentProvider,
  StoreKitPaymentProviderAdapter,
  StoreKitPaymentProviderRegistration,
  StoreKitPaymentStatus,
  StoreKitPaymentLogger,
} from "./types.js";

export class StoreKitPaymentProviderSelectionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "StoreKitPaymentProviderSelectionError";
  }
}

export class StoreKitPaymentStateTransitionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "StoreKitPaymentStateTransitionError";
  }
}

export const STOREKIT_PAYMENT_STATE_MACHINE: ReadonlyArray<{
  from: StoreKitPaymentStatus;
  event: StoreKitPaymentEventType | "cancelPayment" | "refundPayment";
  to: StoreKitPaymentStatus;
  note: string;
}> = [
  { from: "pending", event: "payment_paid", to: "paid", note: "amount, currency, tenant, and reference must match before applying" },
  { from: "pending", event: "payment_failed", to: "failed", note: "provider has finalized failure" },
  { from: "pending", event: "payment_cancelled", to: "cancelled", note: "provider/user cancellation" },
  { from: "pending", event: "cancelPayment", to: "cancelled", note: "only while provider payment is cancellable" },
  { from: "pending", event: "payment_expired", to: "cancelled", note: "expired checkout/session/payment link closes as cancelled" },
  { from: "requires_action", event: "payment_paid", to: "paid", note: "additional authorization succeeded" },
  { from: "requires_action", event: "payment_failed", to: "failed", note: "additional authorization failed or provider finalized failure" },
  { from: "requires_action", event: "payment_cancelled", to: "cancelled", note: "user/provider cancellation" },
  { from: "requires_action", event: "cancelPayment", to: "cancelled", note: "only while provider payment is cancellable" },
  { from: "paid", event: "payment_refunded", to: "refunded", note: "full refund only" },
  { from: "paid", event: "refundPayment", to: "refunded", note: "full refund only" },
  { from: "paid", event: "payment_partially_refunded", to: "paid", note: "partial refunds keep payment paid and require refund records" },
  { from: "paid", event: "payment_dispute_opened", to: "paid", note: "dispute state is tracked outside payment status" },
  { from: "paid", event: "payment_dispute_closed", to: "paid", note: "won/no-change dispute result" },
  { from: "paid", event: "payment_dispute_closed", to: "refunded", note: "lost/refunded dispute result" },
];

export class StoreKitPaymentProviderRegistry {
  private readonly adapters: Partial<Record<StoreKitPaymentProvider, StoreKitPaymentProviderAdapter>> = {};
  private readonly logger?: StoreKitPaymentLogger;
  private readonly deterministicProviderOrder: StoreKitPaymentProvider[] = ["square", "stripe"];

  constructor(registrations: StoreKitPaymentProviderRegistration[] = [], logger?: StoreKitPaymentLogger) {
    this.logger = logger;
    for (const registration of registrations) {
      this.register(registration);
    }
  }

  register(registration: StoreKitPaymentProviderRegistration): void {
    const { adapter, isConfigured = true, unavailableReason } = registration;

    if (!isConfigured || unavailableReason) {
      this.logger?.warn("Skipping unavailable StoreKit payment provider", {
        provider: adapter.provider,
        reason: unavailableReason ?? "missing required provider configuration",
      });
      return;
    }

    this.adapters[adapter.provider] = adapter;
  }

  get(provider: StoreKitPaymentProvider): StoreKitPaymentProviderAdapter | undefined {
    return this.adapters[provider];
  }

  select(priority: StoreKitPaymentProvider[]): StoreKitPaymentProviderAdapter {
    for (const provider of priority) {
      const adapter = this.adapters[provider];
      if (adapter) {
        return adapter;
      }
    }

    throw new StoreKitPaymentProviderSelectionError(
      "No usable StoreKit payment provider matched the explicit provider priority",
    );
  }

  listRegisteredProviders(): StoreKitPaymentProvider[] {
    const providers: StoreKitPaymentProvider[] = [];
    for (const provider of this.deterministicProviderOrder) {
      if (this.adapters[provider]) {
        providers.push(provider);
      }
    }
    return providers;
  }
}

export function transitionStoreKitPaymentStatus(
  current: StoreKitPaymentStatus,
  event: StoreKitPaymentEventType | "cancelPayment" | "refundPayment",
  options: { fullRefund?: boolean; disputeResult?: "won" | "lost_or_refunded" } = {},
): StoreKitPaymentStatus {
  if (event === "payment_unknown") {
    return current;
  }

  if (current === "cancelled" || current === "refunded" || current === "failed") {
    throw new StoreKitPaymentStateTransitionError(
      `StoreKit payment status "${current}" is terminal and cannot transition on "${event}"`,
    );
  }

  if (current === "paid" && event === "payment_partially_refunded") {
    return "paid";
  }

  if (current === "paid" && (event === "payment_refunded" || event === "refundPayment")) {
    if (options.fullRefund === false) {
      return "paid";
    }
    return "refunded";
  }

  if (current === "paid" && event === "payment_dispute_opened") {
    return "paid";
  }

  if (current === "paid" && event === "payment_dispute_closed") {
    return options.disputeResult === "lost_or_refunded" ? "refunded" : "paid";
  }

  const match = STOREKIT_PAYMENT_STATE_MACHINE.find(
    (rule) => rule.from === current && rule.event === event,
  );

  if (!match) {
    throw new StoreKitPaymentStateTransitionError(
      `StoreKit payment status "${current}" cannot transition on "${event}"`,
    );
  }

  return match.to;
}
