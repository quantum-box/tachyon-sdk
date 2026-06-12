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

type SquareMoney = {
  amount?: number;
  currency?: string;
};

type SquarePayment = {
  id?: string;
  status?: string;
  amount_money?: SquareMoney;
  total_money?: SquareMoney;
  refunded_money?: SquareMoney;
  order_id?: string;
  created_at?: string;
  updated_at?: string;
};

type SquarePaymentLink = {
  id?: string;
  url?: string;
  order_id?: string;
  version?: number;
  created_at?: string;
  updated_at?: string;
  checkout_options?: {
    redirect_url?: string;
  };
};

type SquareOrder = {
  id?: string;
  state?: string;
  net_amount_due_money?: SquareMoney;
  total_money?: SquareMoney;
  created_at?: string;
  updated_at?: string;
};

type SquareRefund = {
  id?: string;
  status?: string;
  amount_money?: SquareMoney;
  payment_id?: string;
  created_at?: string;
  updated_at?: string;
};

type SquareDispute = {
  id?: string;
  state?: string;
  payment_id?: string;
  amount_money?: SquareMoney;
  created_at?: string;
  updated_at?: string;
};

type SquareWebhookEvent = {
  merchant_id?: string;
  location_id?: string;
  type?: string;
  event_id?: string;
  created_at?: string;
  data?: {
    id?: string;
    type?: string;
    object?: {
      payment?: SquarePayment;
      payment_link?: SquarePaymentLink;
      order?: SquareOrder;
      refund?: SquareRefund;
      dispute?: SquareDispute;
    };
  };
};

type SquarePaymentLinkCreateInput = {
  idempotencyKey: string;
  amountMinor: number;
  currency: string;
  reference: StoreKitPaymentReference;
  successUrl?: string;
  cancelUrl?: string;
  customer?: StoreKitPaymentCreateInput["customer"];
};

export interface StoreKitSquareClient {
  createPaymentLink(input: SquarePaymentLinkCreateInput): Promise<SquarePaymentLink>;
  getPayment(paymentId: string): Promise<SquarePayment | undefined>;
  cancelPayment(paymentId: string, idempotencyKey: string): Promise<SquarePayment>;
  refundPayment(input: {
    paymentId: string;
    amountMinor?: number;
    reason?: string;
    idempotencyKey: string;
  }): Promise<SquareRefund>;
}

export interface StoreKitSquarePaymentStore {
  save(payment: StoreKitPayment): Promise<StoreKitPayment>;
  getById(paymentId: string): Promise<StoreKitPayment | undefined>;
  getByProviderPaymentId(providerPaymentId: string): Promise<StoreKitPayment | undefined>;
  getByProviderReference(providerReference: string): Promise<StoreKitPayment | undefined>;
}

export interface StoreKitSquareAdapterConfig {
  client: StoreKitSquareClient;
  store: StoreKitSquarePaymentStore;
  webhookSignatureKey: string;
  webhookNotificationUrl: string;
  now?: () => Date;
  idFactory?: () => string;
}

export class StoreKitSquareAdapterError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "StoreKitSquareAdapterError";
  }
}

export class StoreKitSquareWebhookSignatureError extends StoreKitSquareAdapterError {
  constructor() {
    super("Invalid Square webhook signature");
    this.name = "StoreKitSquareWebhookSignatureError";
  }
}

export class StoreKitSquarePaymentNotFoundError extends StoreKitSquareAdapterError {
  constructor(paymentId: string) {
    super(`Square StoreKit payment not found: ${paymentId}`);
    this.name = "StoreKitSquarePaymentNotFoundError";
  }
}

export class StoreKitSquareTenantMismatchError extends StoreKitSquareAdapterError {
  constructor() {
    super("Square StoreKit payment tenant mismatch");
    this.name = "StoreKitSquareTenantMismatchError";
  }
}

export class StoreKitSquareAdapter implements StoreKitPaymentProviderAdapter {
  readonly provider = "square" as const;

  private readonly client: StoreKitSquareClient;
  private readonly store: StoreKitSquarePaymentStore;
  private readonly webhookSignatureKey: string;
  private readonly webhookNotificationUrl: string;
  private readonly now: () => Date;
  private readonly idFactory: () => string;

  constructor(config: StoreKitSquareAdapterConfig) {
    this.client = config.client;
    this.store = config.store;
    this.webhookSignatureKey = config.webhookSignatureKey;
    this.webhookNotificationUrl = config.webhookNotificationUrl;
    this.now = config.now ?? (() => new Date());
    this.idFactory = config.idFactory ?? (() => `sk_square_${Date.now()}`);
  }

  async createPayment(input: StoreKitPaymentCreateInput): Promise<StoreKitPayment> {
    const paymentLink = await this.client.createPaymentLink({
      idempotencyKey: input.idempotencyKey,
      amountMinor: input.amountMinor,
      currency: input.currency,
      reference: input.reference,
      successUrl: input.successUrl,
      cancelUrl: input.cancelUrl,
      customer: input.customer,
    });

    if (!paymentLink.id) {
      throw new StoreKitSquareAdapterError("Square payment link response did not include an id");
    }

    const timestamp = this.isoNow();
    return this.store.save({
      id: this.idFactory(),
      tenantId: input.tenantId,
      provider: "square",
      kind: input.kind,
      status: "pending",
      amountMinor: input.amountMinor,
      currency: input.currency,
      checkoutUrl: paymentLink.url,
      providerPaymentId: paymentLink.id,
      providerReference: paymentLink.order_id,
      idempotencyKey: input.idempotencyKey,
      createdAt: paymentLink.created_at ?? timestamp,
      updatedAt: paymentLink.updated_at ?? timestamp,
    });
  }

  async getPayment(input: StoreKitPaymentGetInput): Promise<StoreKitPayment> {
    const payment = await this.getStoredPayment(input.paymentId, input.tenantId);
    const squarePayment = await this.client.getPayment(payment.providerPaymentId);
    if (!squarePayment) {
      return payment;
    }

    return this.store.save({
      ...payment,
      status: mapSquarePaymentStatus(squarePayment.status) ?? payment.status,
      amountMinor: squarePayment.amount_money?.amount ?? payment.amountMinor,
      currency: squarePayment.amount_money?.currency ?? payment.currency,
      providerReference: squarePayment.order_id ?? payment.providerReference,
      updatedAt: squarePayment.updated_at ?? this.isoNow(),
    });
  }

  async cancelPayment(input: StoreKitPaymentCancelInput): Promise<StoreKitPayment> {
    const payment = await this.getStoredPayment(input.paymentId, input.tenantId);
    const squarePayment = await this.client.cancelPayment(payment.providerPaymentId, input.idempotencyKey);

    return this.store.save({
      ...payment,
      status: mapSquarePaymentStatus(squarePayment.status) ?? "cancelled",
      updatedAt: squarePayment.updated_at ?? this.isoNow(),
    });
  }

  async refundPayment(input: StoreKitRefundInput): Promise<StoreKitPayment> {
    const payment = await this.getStoredPayment(input.paymentId, input.tenantId);
    const refund = await this.client.refundPayment({
      paymentId: payment.providerPaymentId,
      amountMinor: input.amountMinor,
      reason: input.reason,
      idempotencyKey: input.idempotencyKey,
    });

    return this.store.save({
      ...payment,
      status: refund.status === "COMPLETED" && isFullRefund(payment, refund) ? "refunded" : payment.status,
      updatedAt: refund.updated_at ?? this.isoNow(),
    });
  }

  async verifyWebhook(input: StoreKitWebhookVerifyInput): Promise<StoreKitVerifiedWebhook> {
    const signature = getHeader(input.headers, "x-square-hmacsha256-signature");
    if (!signature || !verifySquareSignature({
      signature,
      rawBody: input.rawBody,
      signatureKey: this.webhookSignatureKey,
      notificationUrl: this.webhookNotificationUrl,
    })) {
      throw new StoreKitSquareWebhookSignatureError();
    }

    const event = parseSquareWebhookEvent(input.rawBody);
    const eventId = event.event_id ?? event.data?.id;
    if (!eventId || !event.type) {
      throw new StoreKitSquareAdapterError("Square webhook event is missing event_id or type");
    }

    const payment = await this.resolveStoredPayment(event);
    if (payment && payment.tenantId !== input.tenantId) {
      throw new StoreKitSquareTenantMismatchError();
    }

    return {
      provider: "square",
      tenantId: input.tenantId,
      eventId,
      rawEventType: event.type,
      receivedAt: event.created_at ?? this.isoNow(),
      payment,
      rawEvent: event,
    };
  }

  async normalizeWebhook(input: StoreKitVerifiedWebhook): Promise<StoreKitPaymentEvent> {
    const event = isSquareWebhookEvent(input.rawEvent) ? input.rawEvent : undefined;
    const payment = input.payment
      ? await this.applyWebhookSnapshot(input.payment, event)
      : undefined;

    return {
      provider: "square",
      eventId: input.eventId,
      eventType: mapSquareWebhookEventType(input.rawEventType, event, payment),
      payment,
      receivedAt: input.receivedAt,
      rawEventType: input.rawEventType,
    };
  }

  private async getStoredPayment(paymentId: string, tenantId: string): Promise<StoreKitPayment> {
    const payment = await this.store.getById(paymentId);
    if (!payment) {
      throw new StoreKitSquarePaymentNotFoundError(paymentId);
    }
    if (payment.tenantId !== tenantId) {
      throw new StoreKitSquareTenantMismatchError();
    }
    return payment;
  }

  private async resolveStoredPayment(event: SquareWebhookEvent): Promise<StoreKitPayment | undefined> {
    const paymentId = event.data?.object?.payment?.id
      ?? event.data?.object?.refund?.payment_id
      ?? event.data?.object?.dispute?.payment_id
      ?? event.data?.object?.payment_link?.id;
    if (paymentId) {
      const payment = await this.store.getByProviderPaymentId(paymentId);
      if (payment) {
        return payment;
      }
    }

    const orderId = event.data?.object?.payment?.order_id
      ?? event.data?.object?.payment_link?.order_id
      ?? event.data?.object?.order?.id;
    return orderId ? this.store.getByProviderReference(orderId) : undefined;
  }

  private async applyWebhookSnapshot(
    payment: StoreKitPayment,
    event: SquareWebhookEvent | undefined,
  ): Promise<StoreKitPayment> {
    const squarePayment = event?.data?.object?.payment;
    const squareRefund = event?.data?.object?.refund;
    const squareOrder = event?.data?.object?.order;
    const nextStatus = mapSquareWebhookPaymentStatus(event, payment);
    const nextPayment: StoreKitPayment = {
      ...payment,
      status: nextStatus ?? payment.status,
      providerPaymentId: squarePayment?.id ?? payment.providerPaymentId,
      amountMinor: squarePayment?.amount_money?.amount
        ?? squareOrder?.total_money?.amount
        ?? payment.amountMinor,
      currency: squarePayment?.amount_money?.currency
        ?? squareOrder?.total_money?.currency
        ?? payment.currency,
      providerReference: squarePayment?.order_id ?? squareOrder?.id ?? payment.providerReference,
      updatedAt: squarePayment?.updated_at
        ?? squareRefund?.updated_at
        ?? squareOrder?.updated_at
        ?? this.isoNow(),
    };

    return this.store.save(nextPayment);
  }

  private isoNow(): string {
    return this.now().toISOString();
  }
}

export function verifySquareSignature(input: {
  signature: string;
  rawBody: string;
  signatureKey: string;
  notificationUrl: string;
}): boolean {
  const expected = createHmac("sha256", input.signatureKey)
    .update(`${input.notificationUrl}${input.rawBody}`)
    .digest("base64");

  const expectedBuffer = Buffer.from(expected);
  const actualBuffer = Buffer.from(input.signature);
  return expectedBuffer.length === actualBuffer.length && timingSafeEqual(expectedBuffer, actualBuffer);
}

function parseSquareWebhookEvent(rawBody: string): SquareWebhookEvent {
  try {
    const parsed = JSON.parse(rawBody) as unknown;
    if (!isSquareWebhookEvent(parsed)) {
      throw new StoreKitSquareAdapterError("Square webhook body is not an event object");
    }
    return parsed;
  } catch (error) {
    if (error instanceof StoreKitSquareAdapterError) {
      throw error;
    }
    throw new StoreKitSquareAdapterError("Square webhook body is not valid JSON");
  }
}

function isSquareWebhookEvent(value: unknown): value is SquareWebhookEvent {
  return typeof value === "object" && value !== null;
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

function mapSquarePaymentStatus(status: string | undefined): StoreKitPaymentStatus | undefined {
  switch (status) {
    case "COMPLETED":
      return "paid";
    case "CANCELED":
    case "CANCELLED":
      return "cancelled";
    case "FAILED":
      return "failed";
    case "APPROVED":
      return "requires_action";
    case "PENDING":
      return "pending";
    default:
      return undefined;
  }
}

function mapSquareWebhookPaymentStatus(
  event: SquareWebhookEvent | undefined,
  payment: StoreKitPayment,
): StoreKitPaymentStatus | undefined {
  const eventType = event?.type;
  if (!eventType) {
    return undefined;
  }

  const squarePaymentStatus = mapSquarePaymentStatus(event.data?.object?.payment?.status);
  if (squarePaymentStatus) {
    return squarePaymentStatus;
  }

  const refund = event.data?.object?.refund;
  if (refund?.status === "COMPLETED") {
    return isFullRefund(payment, refund) ? "refunded" : payment.status;
  }

  const orderState = event.data?.object?.order?.state;
  if (orderState === "COMPLETED") {
    return "paid";
  }
  if (orderState === "CANCELED" || orderState === "CANCELLED") {
    return "cancelled";
  }

  if (eventType.includes("payment_link") && (eventType.includes("deleted") || eventType.includes("expired"))) {
    return "cancelled";
  }

  return undefined;
}

function mapSquareWebhookEventType(
  rawEventType: string,
  event: SquareWebhookEvent | undefined,
  payment: StoreKitPayment | undefined,
): StoreKitPaymentEventType {
  const squarePayment = event?.data?.object?.payment;
  const paymentStatus = mapSquarePaymentStatus(squarePayment?.status);
  if (paymentStatus === "paid") {
    return "payment_paid";
  }
  if (paymentStatus === "cancelled") {
    return "payment_cancelled";
  }
  if (paymentStatus === "failed") {
    return "payment_failed";
  }

  const refund = event?.data?.object?.refund;
  if (refund?.status === "COMPLETED") {
    return payment && isFullRefund(payment, refund)
      ? "payment_refunded"
      : "payment_partially_refunded";
  }

  const dispute = event?.data?.object?.dispute;
  if (dispute) {
    return isClosedDisputeState(dispute.state)
      ? "payment_dispute_closed"
      : "payment_dispute_opened";
  }

  const orderState = event?.data?.object?.order?.state;
  if (orderState === "COMPLETED") {
    return "payment_paid";
  }
  if (orderState === "CANCELED" || orderState === "CANCELLED") {
    return "payment_cancelled";
  }

  if (rawEventType.includes("payment_link") && (rawEventType.includes("deleted") || rawEventType.includes("expired"))) {
    return "payment_expired";
  }

  return "payment_unknown";
}

function isFullRefund(payment: StoreKitPayment, refund: SquareRefund): boolean {
  return typeof refund.amount_money?.amount === "number"
    && refund.amount_money.amount >= payment.amountMinor;
}

function isClosedDisputeState(state: string | undefined): boolean {
  return state === "WON"
    || state === "LOST"
    || state === "ACCEPTED"
    || state === "CLOSED";
}
