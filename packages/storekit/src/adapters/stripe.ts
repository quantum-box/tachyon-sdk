import { createHmac, timingSafeEqual } from "node:crypto";
import type {
  StoreKitPayment,
  StoreKitPaymentCancelInput,
  StoreKitPaymentCreateInput,
  StoreKitPaymentEvent,
  StoreKitPaymentEventType,
  StoreKitPaymentGetInput,
  StoreKitPaymentProviderAdapter,
  StoreKitPaymentReference,
  StoreKitPaymentStatus,
  StoreKitRefundInput,
  StoreKitVerifiedWebhook,
  StoreKitWebhookVerifyInput,
} from "../types.js";

type StripeMetadata = Record<string, string | undefined>;

type StripePaymentIntent = {
  id?: string;
  status?: string;
  amount?: number;
  currency?: string;
  metadata?: StripeMetadata;
  created?: number;
  canceled_at?: number | null;
};

type StripeCheckoutSession = {
  id?: string;
  url?: string | null;
  payment_intent?: string | StripePaymentIntent | null;
  payment_status?: string;
  status?: string;
  amount_total?: number | null;
  currency?: string | null;
  metadata?: StripeMetadata;
  created?: number;
  expires_at?: number | null;
};

type StripeRefund = {
  id?: string;
  payment_intent?: string | StripePaymentIntent | null;
  amount?: number;
  currency?: string;
  status?: string;
  created?: number;
};

type StripeCharge = {
  id?: string;
  payment_intent?: string | StripePaymentIntent | null;
  amount?: number;
  amount_refunded?: number;
  currency?: string;
  refunded?: boolean;
  metadata?: StripeMetadata;
  created?: number;
};

type StripeDispute = {
  id?: string;
  payment_intent?: string | StripePaymentIntent | null;
  charge?: string | StripeCharge | null;
  amount?: number;
  currency?: string;
  status?: string;
  metadata?: StripeMetadata;
  created?: number;
};

type StripeWebhookEvent = {
  id?: string;
  type?: string;
  created?: number;
  data?: {
    object?: StripePaymentIntent | StripeCheckoutSession | StripeRefund | StripeCharge | StripeDispute;
  };
};
type StripeWebhookEventObject = NonNullable<StripeWebhookEvent["data"]>["object"];

type StripeCheckoutSessionCreateInput = {
  idempotencyKey: string;
  amountMinor: number;
  currency: string;
  reference: StoreKitPaymentReference;
  successUrl?: string;
  cancelUrl?: string;
  customer?: StoreKitPaymentCreateInput["customer"];
  metadata: StripeMetadata;
};

export interface StoreKitStripeClient {
  createCheckoutSession(input: StripeCheckoutSessionCreateInput): Promise<StripeCheckoutSession>;
  getPaymentIntent(paymentIntentId: string): Promise<StripePaymentIntent | undefined>;
  cancelPaymentIntent(paymentIntentId: string, idempotencyKey: string): Promise<StripePaymentIntent>;
  refundPayment(input: {
    paymentIntentId: string;
    amountMinor?: number;
    reason?: string;
    idempotencyKey: string;
  }): Promise<StripeRefund>;
}

export interface StoreKitStripePaymentStore {
  save(payment: StoreKitPayment): Promise<StoreKitPayment>;
  getById(paymentId: string): Promise<StoreKitPayment | undefined>;
  getByProviderPaymentId(providerPaymentId: string): Promise<StoreKitPayment | undefined>;
  getByProviderReference(providerReference: string): Promise<StoreKitPayment | undefined>;
}

export interface StoreKitStripeAdapterConfig {
  client: StoreKitStripeClient;
  store: StoreKitStripePaymentStore;
  webhookSecret: string;
  webhookToleranceSeconds?: number;
  now?: () => Date;
  idFactory?: () => string;
}

export class StoreKitStripeAdapterError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "StoreKitStripeAdapterError";
  }
}

export class StoreKitStripeWebhookSignatureError extends StoreKitStripeAdapterError {
  constructor() {
    super("Invalid Stripe webhook signature");
    this.name = "StoreKitStripeWebhookSignatureError";
  }
}

export class StoreKitStripePaymentNotFoundError extends StoreKitStripeAdapterError {
  constructor(paymentId: string) {
    super(`Stripe StoreKit payment not found: ${paymentId}`);
    this.name = "StoreKitStripePaymentNotFoundError";
  }
}

export class StoreKitStripeTenantMismatchError extends StoreKitStripeAdapterError {
  constructor() {
    super("Stripe StoreKit payment tenant mismatch");
    this.name = "StoreKitStripeTenantMismatchError";
  }
}

export class StoreKitStripeAdapter implements StoreKitPaymentProviderAdapter {
  readonly provider = "stripe" as const;

  private readonly client: StoreKitStripeClient;
  private readonly store: StoreKitStripePaymentStore;
  private readonly webhookSecret: string;
  private readonly webhookToleranceSeconds: number;
  private readonly now: () => Date;
  private readonly idFactory: () => string;

  constructor(config: StoreKitStripeAdapterConfig) {
    this.client = config.client;
    this.store = config.store;
    this.webhookSecret = config.webhookSecret;
    this.webhookToleranceSeconds = config.webhookToleranceSeconds ?? 300;
    this.now = config.now ?? (() => new Date());
    this.idFactory = config.idFactory ?? (() => `sk_stripe_${Date.now()}`);
  }

  async createPayment(input: StoreKitPaymentCreateInput): Promise<StoreKitPayment> {
    const paymentId = this.idFactory();
    const session = await this.client.createCheckoutSession({
      idempotencyKey: input.idempotencyKey,
      amountMinor: input.amountMinor,
      currency: input.currency,
      reference: input.reference,
      successUrl: input.successUrl,
      cancelUrl: input.cancelUrl,
      customer: input.customer,
      metadata: buildStripeMetadata(paymentId, input),
    });

    if (!session.id) {
      throw new StoreKitStripeAdapterError("Stripe checkout session response did not include an id");
    }

    const paymentIntentId = getPaymentIntentId(session.payment_intent);
    const timestamp = this.isoNow();
    return this.store.save({
      id: paymentId,
      tenantId: input.tenantId,
      provider: "stripe",
      kind: input.kind,
      status: mapStripeCheckoutSessionStatus(session) ?? "pending",
      amountMinor: session.amount_total ?? input.amountMinor,
      currency: session.currency?.toUpperCase() ?? input.currency,
      checkoutUrl: session.url ?? undefined,
      providerPaymentId: paymentIntentId ?? session.id,
      providerReference: session.id,
      idempotencyKey: input.idempotencyKey,
      createdAt: fromStripeTimestamp(session.created) ?? timestamp,
      updatedAt: timestamp,
    });
  }

  async getPayment(input: StoreKitPaymentGetInput): Promise<StoreKitPayment> {
    const payment = await this.getStoredPayment(input.paymentId, input.tenantId);
    const paymentIntent = await this.client.getPaymentIntent(payment.providerPaymentId);
    if (!paymentIntent) {
      return payment;
    }

    return this.store.save({
      ...payment,
      status: mapStripePaymentIntentStatus(paymentIntent.status) ?? payment.status,
      amountMinor: paymentIntent.amount ?? payment.amountMinor,
      currency: paymentIntent.currency?.toUpperCase() ?? payment.currency,
      updatedAt: this.isoNow(),
    });
  }

  async cancelPayment(input: StoreKitPaymentCancelInput): Promise<StoreKitPayment> {
    const payment = await this.getStoredPayment(input.paymentId, input.tenantId);
    const paymentIntent = await this.client.cancelPaymentIntent(payment.providerPaymentId, input.idempotencyKey);

    return this.store.save({
      ...payment,
      status: mapStripePaymentIntentStatus(paymentIntent.status) ?? "cancelled",
      updatedAt: fromStripeTimestamp(paymentIntent.canceled_at ?? undefined) ?? this.isoNow(),
    });
  }

  async refundPayment(input: StoreKitRefundInput): Promise<StoreKitPayment> {
    const payment = await this.getStoredPayment(input.paymentId, input.tenantId);
    const refund = await this.client.refundPayment({
      paymentIntentId: payment.providerPaymentId,
      amountMinor: input.amountMinor,
      reason: input.reason,
      idempotencyKey: input.idempotencyKey,
    });

    return this.store.save({
      ...payment,
      status: refund.status === "succeeded" && isFullRefund(payment, refund) ? "refunded" : payment.status,
      updatedAt: fromStripeTimestamp(refund.created) ?? this.isoNow(),
    });
  }

  async verifyWebhook(input: StoreKitWebhookVerifyInput): Promise<StoreKitVerifiedWebhook> {
    const signature = getHeader(input.headers, "stripe-signature");
    if (!signature || !verifyStripeSignature({
      signature,
      rawBody: input.rawBody,
      webhookSecret: this.webhookSecret,
      toleranceSeconds: this.webhookToleranceSeconds,
      now: this.now,
    })) {
      throw new StoreKitStripeWebhookSignatureError();
    }

    const event = parseStripeWebhookEvent(input.rawBody);
    if (!event.id || !event.type) {
      throw new StoreKitStripeAdapterError("Stripe webhook event is missing id or type");
    }

    const payment = await this.resolveStoredPayment(event);
    if (payment && payment.tenantId !== input.tenantId) {
      throw new StoreKitStripeTenantMismatchError();
    }

    return {
      provider: "stripe",
      tenantId: input.tenantId,
      eventId: event.id,
      rawEventType: event.type,
      receivedAt: fromStripeTimestamp(event.created) ?? this.isoNow(),
      payment,
      rawEvent: event,
    };
  }

  async normalizeWebhook(input: StoreKitVerifiedWebhook): Promise<StoreKitPaymentEvent> {
    const event = isStripeWebhookEvent(input.rawEvent) ? input.rawEvent : undefined;
    const payment = input.payment
      ? await this.applyWebhookSnapshot(input.payment, event)
      : undefined;

    return {
      provider: "stripe",
      eventId: input.eventId,
      eventType: mapStripeWebhookEventType(input.rawEventType, event, payment),
      payment,
      receivedAt: input.receivedAt,
      rawEventType: input.rawEventType,
    };
  }

  private async getStoredPayment(paymentId: string, tenantId: string): Promise<StoreKitPayment> {
    const payment = await this.store.getById(paymentId);
    if (!payment) {
      throw new StoreKitStripePaymentNotFoundError(paymentId);
    }
    if (payment.tenantId !== tenantId) {
      throw new StoreKitStripeTenantMismatchError();
    }
    return payment;
  }

  private async resolveStoredPayment(event: StripeWebhookEvent): Promise<StoreKitPayment | undefined> {
    const object = event.data?.object;
    const metadataPaymentId = object && "metadata" in object
      ? object.metadata?.storekitPaymentId
      : undefined;
    if (metadataPaymentId) {
      const payment = await this.store.getById(metadataPaymentId);
      if (payment) {
        return payment;
      }
    }

    const paymentIntentId = getObjectPaymentIntentId(object);
    if (paymentIntentId) {
      const payment = await this.store.getByProviderPaymentId(paymentIntentId);
      if (payment) {
        return payment;
      }
    }

    const checkoutSessionId = isStripeCheckoutSession(object) ? object.id : undefined;
    return checkoutSessionId ? this.store.getByProviderReference(checkoutSessionId) : undefined;
  }

  private async applyWebhookSnapshot(
    payment: StoreKitPayment,
    event: StripeWebhookEvent | undefined,
  ): Promise<StoreKitPayment> {
    const object = event?.data?.object;
    const paymentIntent = isStripePaymentIntent(object) ? object : undefined;
    const checkoutSession = isStripeCheckoutSession(object) ? object : undefined;
    const charge = isStripeCharge(object) ? object : undefined;
    const nextStatus = mapStripeWebhookPaymentStatus(event, payment);
    const paymentIntentId = getObjectPaymentIntentId(object);

    return this.store.save({
      ...payment,
      status: nextStatus ?? payment.status,
      providerPaymentId: paymentIntentId ?? payment.providerPaymentId,
      providerReference: checkoutSession?.id ?? payment.providerReference,
      amountMinor: paymentIntent?.amount
        ?? checkoutSession?.amount_total
        ?? charge?.amount
        ?? payment.amountMinor,
      currency: paymentIntent?.currency?.toUpperCase()
        ?? checkoutSession?.currency?.toUpperCase()
        ?? charge?.currency?.toUpperCase()
        ?? payment.currency,
      updatedAt: fromStripeTimestamp(event?.created) ?? this.isoNow(),
    });
  }

  private isoNow(): string {
    return this.now().toISOString();
  }
}

export function verifyStripeSignature(input: {
  signature: string;
  rawBody: string;
  webhookSecret: string;
  toleranceSeconds?: number;
  now?: () => Date;
}): boolean {
  const timestamp = getStripeSignaturePart(input.signature, "t");
  const signatures = getStripeSignatureParts(input.signature, "v1");
  if (!timestamp || signatures.length === 0) {
    return false;
  }

  const timestampSeconds = Number(timestamp);
  if (!Number.isFinite(timestampSeconds)) {
    return false;
  }

  const toleranceSeconds = input.toleranceSeconds ?? 300;
  const nowSeconds = Math.floor((input.now?.() ?? new Date()).getTime() / 1000);
  if (Math.abs(nowSeconds - timestampSeconds) > toleranceSeconds) {
    return false;
  }

  const expected = createHmac("sha256", input.webhookSecret)
    .update(`${timestamp}.${input.rawBody}`)
    .digest("hex");

  return signatures.some((signature) => safeEqual(expected, signature));
}

function buildStripeMetadata(paymentId: string, input: StoreKitPaymentCreateInput): StripeMetadata {
  return {
    storekitPaymentId: paymentId,
    storekitTenantId: input.tenantId,
    storekitKind: input.kind,
    invoiceId: input.reference.invoiceId,
    reservationId: input.reference.reservationId,
    fieldPaymentIntentId: input.reference.fieldPaymentIntentId,
    orderId: input.reference.orderId,
  };
}

function parseStripeWebhookEvent(rawBody: string): StripeWebhookEvent {
  try {
    const parsed = JSON.parse(rawBody) as unknown;
    if (!isStripeWebhookEvent(parsed)) {
      throw new StoreKitStripeAdapterError("Stripe webhook body is not an event object");
    }
    return parsed;
  } catch (error) {
    if (error instanceof StoreKitStripeAdapterError) {
      throw error;
    }
    throw new StoreKitStripeAdapterError("Stripe webhook body is not valid JSON");
  }
}

function isStripeWebhookEvent(value: unknown): value is StripeWebhookEvent {
  return typeof value === "object" && value !== null;
}

function isStripePaymentIntent(value: unknown): value is StripePaymentIntent {
  return typeof value === "object"
    && value !== null
    && "status" in value
    && !("payment_status" in value)
    && !("payment_intent" in value)
    && !("amount_refunded" in value)
    && !("charge" in value);
}

function isStripeCheckoutSession(value: unknown): value is StripeCheckoutSession {
  return typeof value === "object" && value !== null && ("payment_status" in value || "expires_at" in value || "url" in value);
}

function isStripeRefund(value: unknown): value is StripeRefund {
  return typeof value === "object" && value !== null && "status" in value && "payment_intent" in value && "amount" in value;
}

function isStripeCharge(value: unknown): value is StripeCharge {
  return typeof value === "object" && value !== null && ("amount_refunded" in value || "refunded" in value);
}

function isStripeDispute(value: unknown): value is StripeDispute {
  return typeof value === "object" && value !== null && "status" in value && ("charge" in value || "payment_intent" in value);
}

function getHeader(headers: Record<string, string>, name: string): string | undefined {
  const target = name.toLowerCase();
  for (const [key, value] of Object.entries(headers)) {
    if (key.toLowerCase() === target) {
      return value;
    }
  }
  return undefined;
}

function getStripeSignaturePart(signature: string, key: string): string | undefined {
  return getStripeSignatureParts(signature, key)[0];
}

function getStripeSignatureParts(signature: string, key: string): string[] {
  return signature
    .split(",")
    .map((part) => part.split("="))
    .filter(([partKey, value]) => partKey === key && Boolean(value))
    .map(([, value]) => value);
}

function safeEqual(expected: string, actual: string): boolean {
  const expectedBuffer = Buffer.from(expected);
  const actualBuffer = Buffer.from(actual);
  return expectedBuffer.length === actualBuffer.length && timingSafeEqual(expectedBuffer, actualBuffer);
}

function getObjectPaymentIntentId(object: StripeWebhookEventObject): string | undefined {
  if (!object) {
    return undefined;
  }
  if (isStripePaymentIntent(object)) {
    return object.id;
  }
  if (isStripeCheckoutSession(object) || isStripeRefund(object) || isStripeCharge(object) || isStripeDispute(object)) {
    return getPaymentIntentId(object.payment_intent);
  }
  return undefined;
}

function getPaymentIntentId(value: string | StripePaymentIntent | null | undefined): string | undefined {
  if (typeof value === "string") {
    return value;
  }
  return value?.id;
}

function mapStripePaymentIntentStatus(status: string | undefined): StoreKitPaymentStatus | undefined {
  switch (status) {
    case "succeeded":
      return "paid";
    case "canceled":
      return "cancelled";
    case "requires_action":
    case "requires_confirmation":
    case "requires_capture":
      return "requires_action";
    case "requires_payment_method":
      return "failed";
    case "processing":
      return "pending";
    default:
      return undefined;
  }
}

function mapStripeCheckoutSessionStatus(session: StripeCheckoutSession): StoreKitPaymentStatus | undefined {
  if (session.payment_status === "paid") {
    return "paid";
  }
  if (session.status === "expired") {
    return "cancelled";
  }
  if (session.status === "open") {
    return "pending";
  }
  return undefined;
}

function mapStripeWebhookPaymentStatus(
  event: StripeWebhookEvent | undefined,
  payment: StoreKitPayment,
): StoreKitPaymentStatus | undefined {
  const object = event?.data?.object;
  if (isStripePaymentIntent(object)) {
    return mapStripePaymentIntentStatus(object.status);
  }
  if (isStripeCheckoutSession(object)) {
    return mapStripeCheckoutSessionStatus(object);
  }
  if (isStripeRefund(object) && object.status === "succeeded") {
    return isFullRefund(payment, object) ? "refunded" : payment.status;
  }
  if (isStripeCharge(object) && object.refunded) {
    return typeof object.amount_refunded === "number" && object.amount_refunded >= payment.amountMinor
      ? "refunded"
      : payment.status;
  }
  return undefined;
}

function mapStripeWebhookEventType(
  rawEventType: string,
  event: StripeWebhookEvent | undefined,
  payment: StoreKitPayment | undefined,
): StoreKitPaymentEventType {
  const object = event?.data?.object;
  if (isStripePaymentIntent(object)) {
    const status = mapStripePaymentIntentStatus(object.status);
    if (status === "paid") {
      return "payment_paid";
    }
    if (status === "cancelled") {
      return "payment_cancelled";
    }
    if (status === "failed") {
      return "payment_failed";
    }
  }

  if (isStripeCheckoutSession(object)) {
    if (object.payment_status === "paid") {
      return "payment_paid";
    }
    if (object.status === "expired" || rawEventType === "checkout.session.expired") {
      return "payment_expired";
    }
  }

  if (isStripeRefund(object) && object.status === "succeeded") {
    return payment && isFullRefund(payment, object)
      ? "payment_refunded"
      : "payment_partially_refunded";
  }

  if (isStripeCharge(object) && (object.refunded || rawEventType.includes("refund"))) {
    return payment && typeof object.amount_refunded === "number" && object.amount_refunded >= payment.amountMinor
      ? "payment_refunded"
      : "payment_partially_refunded";
  }

  if (rawEventType.startsWith("charge.dispute.") || isStripeDispute(object)) {
    return isClosedDisputeStatus(isStripeDispute(object) ? object.status : undefined, rawEventType)
      ? "payment_dispute_closed"
      : "payment_dispute_opened";
  }

  if (rawEventType === "payment_intent.payment_failed") {
    return "payment_failed";
  }

  return "payment_unknown";
}

function isFullRefund(payment: StoreKitPayment, refund: StripeRefund): boolean {
  return typeof refund.amount === "number" && refund.amount >= payment.amountMinor;
}

function isClosedDisputeStatus(status: string | undefined, rawEventType: string): boolean {
  return rawEventType === "charge.dispute.closed"
    || status === "won"
    || status === "lost"
    || status === "warning_closed";
}

function fromStripeTimestamp(timestamp: number | null | undefined): string | undefined {
  return typeof timestamp === "number"
    ? new Date(timestamp * 1000).toISOString()
    : undefined;
}
