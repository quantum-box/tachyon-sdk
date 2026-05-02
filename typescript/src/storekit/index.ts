import type { FetchAPI, HTTPHeaders } from '../runtime';

export type StoreKitMoney = {
    nanodollar: number;
    currency?: string;
    minorUnit?: number;
};

export type StoreKitList<T> = {
    object: 'list';
    url: string;
    hasMore: boolean;
    data: T[];
};

export type StoreKitAddress = {
    postalCode: string;
    state: string;
    city: string;
    address1: string;
    address2?: string | null;
};

export type StoreKitProduct = {
    object: 'product';
    id: string;
    name: string;
    description?: string | null;
    kind?: string;
    listPrice?: number;
    billingCycle?: string;
    publicationName?: string | null;
    publicationDescription?: string | null;
    imageIds: string[];
    categoryId?: string | null;
    weightGrams?: number | null;
    created?: number;
};

export type StoreKitCategory = {
    object: 'product_category';
    id: string;
    name: string;
    slug?: string | null;
    parentId?: string | null;
    sortOrder?: number;
    imageUrl?: string | null;
};

export type StoreKitProductStock = {
    object: 'stock';
    id: string;
    productId: string;
    quantityOnHand: number;
    quantityReserved: number;
    quantityAvailable: number;
    lowStockThreshold: number;
    trackInventory: boolean;
    createdAt: string;
    updatedAt: string;
    created?: number;
};

export type StoreKitCartItem = {
    object: 'cart_item';
    id: string;
    productId: string;
    quantity: number;
    unitPrice: StoreKitMoney;
};

export type StoreKitCart = {
    object: 'cart';
    id: string;
    tenantId: string;
    userId?: string | null;
    sessionId?: string | null;
    status: string;
    items: StoreKitCartItem[];
    expiresAt?: string | null;
    createdAt: string;
    updatedAt: string;
    created?: number;
};

export type StoreKitCoupon = {
    object: 'coupon';
    id: string;
    tenantId: string;
    code: string;
    discountType: string;
    discountValue: number;
    currency: string;
    isActive: boolean;
    expiresAt?: string | null;
    usageLimit?: number | null;
    usedCount: number;
    minimumOrderAmount?: number | null;
    usePerUser: boolean;
    discountAmount?: number | null;
    createdAt?: string;
    updatedAt?: string;
    created?: number;
};

export type StoreKitOrderItem = {
    object: 'order_item';
    id: string;
    productId: string;
    productName: string;
    quantity: number;
    unitPrice: StoreKitMoney;
    subtotal: StoreKitMoney;
};

export type StoreKitOrder = {
    object: 'order';
    id: string;
    tenantId: string;
    cartId?: string | null;
    userId?: string | null;
    sessionId?: string | null;
    status: string;
    fulfillmentMethod?: string | null;
    paymentMethod?: string | null;
    shippingName?: string | null;
    shippingAddress?: string | null;
    shippingPhone?: string | null;
    subtotal: StoreKitMoney;
    discount: StoreKitMoney;
    shippingFee: StoreKitMoney;
    total: StoreKitMoney;
    items: StoreKitOrderItem[];
    confirmedAt?: string | null;
    shippedAt?: string | null;
    deliveredAt?: string | null;
    cancelledAt?: string | null;
    pickupRequestedAt?: string | null;
    pickupDeadline?: string | null;
    readyAt?: string | null;
    pickedUpAt?: string | null;
    refundedAt?: string | null;
    paymentStatus?: string | null;
    checkoutUrl?: string | null;
    createdAt: string;
    updatedAt?: string;
    created?: number;
};

export type StoreKitCustomer = {
    object: 'customer';
    id: string;
    name: string;
    email: string;
    phone?: string | null;
    description?: string | null;
};

export type StoreKitDeleted = {
    id: string;
    object: string;
    deleted: boolean;
};

export type StoreKitClientOptions = {
    baseUrl: string;
    operatorId: string;
    publicApiKey?: string;
    sessionId?: string;
    userId?: string;
    fetchApi?: FetchAPI;
    headers?: HTTPHeaders;
};

export type StoreKitRequestOptions = {
    idempotencyKey?: string;
    headers?: HTTPHeaders;
};

export type StoreKitPaginationParams = {
    limit?: number;
    offset?: number;
    startingAfter?: string;
    endingBefore?: string;
};

export type ListProductsParams = StoreKitPaginationParams & {
    categoryId?: string;
    search?: string;
};

export type ListOrdersParams = StoreKitPaginationParams & {
    userId?: string;
    sessionId?: string;
};

export type CreateCartParams = {
    userId?: string;
    sessionId?: string;
};

export type AddCartItemParams = {
    cartId: string;
    productId: string;
    quantity: number;
};

export type UpdateCartItemParams = {
    cartId: string;
    itemId: string;
    quantity: number;
};

export type RemoveCartItemParams = {
    cartId: string;
    itemId: string;
};

export type ValidateCouponParams = {
    code: string;
    subtotalNanodollar?: number;
};

export type CheckoutParams = {
    cartId: string;
    shippingName?: string;
    shippingAddress?: string;
    shippingPhone?: string;
    customerEmail?: string;
    storeId?: string;
    pickupRequestedAt?: string;
    fulfillmentMethod?: 'delivery' | 'pickup' | string;
    paymentMethod?: 'online' | 'cash_on_pickup' | string;
    couponCode?: string;
    successUrl?: string;
    cancelUrl?: string;
};

export type CreateCustomerParams = {
    name: string;
    email: string;
    phone?: string;
    address?: StoreKitAddress;
};

export type UpdateCustomerParams = {
    name?: string;
    email?: string;
    phone?: string;
    address?: StoreKitAddress;
};

export class StoreKitError extends Error {
    override name = 'StoreKitError';
    readonly type: string;
    readonly code?: string;
    readonly param?: string;
    readonly status: number;
    readonly response?: Response;

    constructor(params: {
        message: string;
        type?: string;
        code?: string;
        param?: string;
        status?: number;
        response?: Response;
    }) {
        super(params.message);
        this.type = params.type ?? 'api_error';
        this.code = params.code;
        this.param = params.param;
        this.status = params.status ?? 0;
        this.response = params.response;
    }

    static async fromResponse(response: Response): Promise<StoreKitError> {
        const fallback = `${response.status} ${response.statusText}`.trim();
        try {
            const payload = await response.clone().json();
            const error = payload?.error ?? payload;
            return new StoreKitError({
                message: String(error?.message ?? fallback),
                type: String(error?.type ?? 'api_error'),
                code: error?.code,
                param: error?.param,
                status: response.status,
                response,
            });
        } catch (_e) {
            const text = await response.clone().text().catch(() => '');
            return new StoreKitError({
                message: text || fallback,
                status: response.status,
                response,
            });
        }
    }
}

export function createStoreKitClient(
    options: StoreKitClientOptions,
): StoreKitClient {
    return new StoreKitClient(options);
}

export function formatNanodollarAsUsd(
    nanodollar: number,
    fractionDigits = 2,
): string {
    return (nanodollar / 1_000_000_000).toLocaleString('en-US', {
        style: 'currency',
        currency: 'USD',
        minimumFractionDigits: fractionDigits,
        maximumFractionDigits: fractionDigits,
    });
}

export function nanodollarToMinorUnit(nanodollar: number): number {
    return Math.round(nanodollar / 10_000_000);
}

export class StoreKitClient {
    readonly products: StoreKitProductsResource;
    readonly cart: StoreKitCartResource;
    readonly coupons: StoreKitCouponsResource;
    readonly checkout: StoreKitCheckoutResource;
    readonly orders: StoreKitOrdersResource;
    readonly customers: StoreKitCustomersResource;

    private readonly baseUrl: string;
    private readonly fetchApi: FetchAPI;

    constructor(private readonly options: StoreKitClientOptions) {
        if (!options.baseUrl) {
            throw new StoreKitError({
                message: 'StoreKit baseUrl is required',
                type: 'invalid_request_error',
                param: 'baseUrl',
            });
        }
        if (!options.operatorId) {
            throw new StoreKitError({
                message: 'StoreKit operatorId is required',
                type: 'invalid_request_error',
                param: 'operatorId',
            });
        }

        this.baseUrl = options.baseUrl.replace(/\/+$/, '');
        this.fetchApi = options.fetchApi ?? fetch;
        this.products = new StoreKitProductsResource(this);
        this.cart = new StoreKitCartResource(this);
        this.coupons = new StoreKitCouponsResource(this);
        this.checkout = new StoreKitCheckoutResource(this);
        this.orders = new StoreKitOrdersResource(this);
        this.customers = new StoreKitCustomersResource(this);
    }

    async request<T>(
        method: string,
        path: string,
        params: {
            query?: Record<string, unknown>;
            body?: unknown;
            options?: StoreKitRequestOptions;
        } = {},
    ): Promise<T> {
        const headers: HTTPHeaders = {
            Accept: 'application/json',
            'x-operator-id': this.options.operatorId,
            ...(this.options.headers ?? {}),
            ...(params.options?.headers ?? {}),
        };

        if (this.options.publicApiKey && !headers.Authorization) {
            headers.Authorization = this.options.publicApiKey.startsWith(
                'Bearer ',
            )
                ? this.options.publicApiKey
                : `Bearer ${this.options.publicApiKey}`;
        }

        if (params.body !== undefined) {
            headers['Content-Type'] = 'application/json';
        }

        if (params.options?.idempotencyKey) {
            headers['Idempotency-Key'] = params.options.idempotencyKey;
        }

        const query = toQueryString(params.query);
        const response = await this.fetchApi(
            `${this.baseUrl}${path}${query ? `?${query}` : ''}`,
            {
                method,
                headers,
                body:
                    params.body === undefined
                        ? undefined
                        : JSON.stringify(params.body),
            },
        );

        if (!response.ok) {
            throw await StoreKitError.fromResponse(response);
        }

        if (response.status === 204) {
            return undefined as T;
        }

        return (await response.json()) as T;
    }

    defaultSessionId(): string | undefined {
        return this.options.sessionId;
    }

    defaultUserId(): string | undefined {
        return this.options.userId;
    }
}

export class StoreKitProductsResource {
    constructor(private readonly client: StoreKitClient) {}

    async list(params: ListProductsParams = {}): Promise<StoreKitList<StoreKitProduct>> {
        const response = await this.client.request<RawList<RawProduct>>(
            'GET',
            '/v1/storekit/products',
            {
                query: {
                    category_id: params.categoryId,
                    search: params.search,
                    limit: params.limit,
                    offset: params.offset,
                    starting_after: params.startingAfter,
                    ending_before: params.endingBefore,
                },
            },
        );
        return mapList(response, mapProduct);
    }

    async get(productId: string): Promise<StoreKitProduct> {
        const response = await this.client.request<RawProduct>(
            'GET',
            `/v1/storekit/products/${encodeURIComponent(productId)}`,
        );
        return mapProduct(response);
    }

    async listCategories(
        params: StoreKitPaginationParams = {},
    ): Promise<StoreKitList<StoreKitCategory>> {
        const response = await this.client.request<RawList<RawCategory>>(
            'GET',
            '/v1/storekit/categories',
            {
                query: {
                    limit: params.limit,
                    offset: params.offset,
                    starting_after: params.startingAfter,
                    ending_before: params.endingBefore,
                },
            },
        );
        return mapList(response, mapCategory);
    }

    async getStock(productId: string): Promise<StoreKitProductStock> {
        const response = await this.client.request<RawStock>(
            'GET',
            `/v1/storekit/products/${encodeURIComponent(productId)}/stock`,
        );
        return mapStock(response);
    }
}

export class StoreKitCartResource {
    constructor(private readonly client: StoreKitClient) {}

    async create(
        params: CreateCartParams = {},
        options?: StoreKitRequestOptions,
    ): Promise<StoreKitCart> {
        const response = await this.client.request<RawCart>(
            'POST',
            '/v1/storekit/carts',
            {
                body: {
                    user_id: params.userId ?? this.client.defaultUserId(),
                    session_id:
                        params.sessionId ?? this.client.defaultSessionId(),
                },
                options,
            },
        );
        return mapCart(response);
    }

    async get(cartId: string): Promise<StoreKitCart> {
        const response = await this.client.request<RawCart>(
            'GET',
            `/v1/storekit/carts/${encodeURIComponent(cartId)}`,
        );
        return mapCart(response);
    }

    async addItem(
        params: AddCartItemParams,
        options?: StoreKitRequestOptions,
    ): Promise<StoreKitCart> {
        const response = await this.client.request<RawCart>(
            'POST',
            `/v1/storekit/carts/${encodeURIComponent(
                params.cartId,
            )}/items`,
            {
                body: {
                    product_id: params.productId,
                    quantity: params.quantity,
                },
                options,
            },
        );
        return mapCart(response);
    }

    async updateItem(
        params: UpdateCartItemParams,
        options?: StoreKitRequestOptions,
    ): Promise<StoreKitCart> {
        const response = await this.client.request<RawCart>(
            'POST',
            `/v1/storekit/carts/${encodeURIComponent(
                params.cartId,
            )}/items/${encodeURIComponent(params.itemId)}`,
            {
                body: { quantity: params.quantity },
                options,
            },
        );
        return mapCart(response);
    }

    async removeItem(params: RemoveCartItemParams): Promise<{ ok: boolean }> {
        return await this.client.request<{ ok: boolean }>(
            'DELETE',
            `/v1/storekit/carts/${encodeURIComponent(
                params.cartId,
            )}/items/${encodeURIComponent(params.itemId)}`,
        );
    }

    async clear(cartId: string): Promise<{ ok: boolean }> {
        return await this.client.request<{ ok: boolean }>(
            'POST',
            `/v1/storekit/carts/${encodeURIComponent(cartId)}/clear`,
        );
    }
}

export class StoreKitCouponsResource {
    constructor(private readonly client: StoreKitClient) {}

    async validate(params: ValidateCouponParams): Promise<StoreKitCoupon> {
        const response = await this.client.request<RawCoupon>(
            'POST',
            '/v1/storekit/coupons/validate',
            {
                body: {
                    code: params.code,
                    subtotal_nanodollar: params.subtotalNanodollar,
                },
            },
        );
        return mapCoupon(response);
    }
}

export class StoreKitCheckoutResource {
    constructor(private readonly client: StoreKitClient) {}

    async create(
        params: CheckoutParams,
        options?: StoreKitRequestOptions,
    ): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>('POST', '/v1/storekit/checkout_sessions', {
            body: {
                cart_id: params.cartId,
                shipping_name: params.shippingName,
                shipping_address: params.shippingAddress,
                shipping_phone: params.shippingPhone,
                customer_email: params.customerEmail,
                store_id: params.storeId,
                pickup_requested_at: params.pickupRequestedAt,
                fulfillment_method: params.fulfillmentMethod,
                payment_method: params.paymentMethod,
                coupon_code: params.couponCode,
                success_url: params.successUrl,
                cancel_url: params.cancelUrl,
            },
            options,
        });
        return mapOrder(response);
    }

    async confirm(orderId: string): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'POST',
            `/v1/storekit/checkout_sessions/${encodeURIComponent(
                orderId,
            )}/confirm`,
        );
        return mapOrder(response);
    }
}

export class StoreKitOrdersResource {
    constructor(private readonly client: StoreKitClient) {}

    async list(params: ListOrdersParams = {}): Promise<StoreKitList<StoreKitOrder>> {
        const response = await this.client.request<RawList<RawOrder>>(
            'GET',
            '/v1/storekit/orders',
            {
                query: {
                    user_id: params.userId ?? this.client.defaultUserId(),
                    session_id:
                        params.sessionId ?? this.client.defaultSessionId(),
                    limit: params.limit,
                    offset: params.offset,
                    starting_after: params.startingAfter,
                    ending_before: params.endingBefore,
                },
            },
        );
        return mapList(response, mapOrder);
    }

    async get(orderId: string): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'GET',
            `/v1/storekit/orders/${encodeURIComponent(orderId)}`,
        );
        return mapOrder(response);
    }

    async cancel(orderId: string): Promise<{ ok: boolean }> {
        return await this.client.request<{ ok: boolean }>(
            'POST',
            `/v1/storekit/orders/${encodeURIComponent(orderId)}/cancel`,
        );
    }

    async selectPickupDatetime(params: {
        orderId: string;
        pickupRequestedAt?: string | null;
    }): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'POST',
            `/v1/storekit/orders/${encodeURIComponent(
                params.orderId,
            )}/select-pickup-datetime`,
            {
                body: {
                    pickup_requested_at: params.pickupRequestedAt,
                },
            },
        );
        return mapOrder(response);
    }

    async prepare(orderId: string): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'POST',
            `/v1/storekit/orders/${encodeURIComponent(orderId)}/prepare`,
        );
        return mapOrder(response);
    }

    async ship(orderId: string): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'POST',
            `/v1/storekit/orders/${encodeURIComponent(orderId)}/ship`,
        );
        return mapOrder(response);
    }

    async deliver(orderId: string): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'POST',
            `/v1/storekit/orders/${encodeURIComponent(orderId)}/deliver`,
        );
        return mapOrder(response);
    }

    async ready(orderId: string): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'POST',
            `/v1/storekit/orders/${encodeURIComponent(orderId)}/ready`,
        );
        return mapOrder(response);
    }

    async pickup(orderId: string): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'POST',
            `/v1/storekit/orders/${encodeURIComponent(orderId)}/pickup`,
        );
        return mapOrder(response);
    }

    async refund(orderId: string): Promise<StoreKitOrder> {
        const response = await this.client.request<RawOrder>(
            'POST',
            `/v1/storekit/orders/${encodeURIComponent(orderId)}/refund`,
        );
        return mapOrder(response);
    }
}

export class StoreKitCustomersResource {
    constructor(private readonly client: StoreKitClient) {}

    async create(
        params: CreateCustomerParams,
        options?: StoreKitRequestOptions,
    ): Promise<StoreKitCustomer> {
        const response = await this.client.request<RawCustomer>(
            'POST',
            '/v1/storekit/customers',
            {
                body: {
                    name: params.name,
                    email: params.email,
                    phone: params.phone,
                    address: mapAddressToRaw(params.address),
                },
                options,
            },
        );
        return mapCustomer(response);
    }

    async list(params: {
        email?: string;
        limit?: number;
    } = {}): Promise<StoreKitList<StoreKitCustomer>> {
        const response = await this.client.request<RawList<RawCustomer>>(
            'GET',
            '/v1/storekit/customers',
            { query: params },
        );
        return mapList(response, mapCustomer);
    }

    async get(customerId: string): Promise<StoreKitCustomer> {
        const response = await this.client.request<RawCustomer>(
            'GET',
            `/v1/storekit/customers/${encodeURIComponent(customerId)}`,
        );
        return mapCustomer(response);
    }

    async update(
        customerId: string,
        params: UpdateCustomerParams,
    ): Promise<StoreKitCustomer> {
        const response = await this.client.request<RawCustomer>(
            'POST',
            `/v1/storekit/customers/${encodeURIComponent(customerId)}`,
            {
                body: {
                    name: params.name,
                    email: params.email,
                    phone: params.phone,
                    address: mapAddressToRaw(params.address),
                },
            },
        );
        return mapCustomer(response);
    }

    async delete(customerId: string): Promise<StoreKitDeleted> {
        return await this.client.request<StoreKitDeleted>(
            'DELETE',
            `/v1/storekit/customers/${encodeURIComponent(customerId)}`,
        );
    }
}

type RawList<T> = {
    object?: 'list';
    url?: string;
    has_more?: boolean;
    hasMore?: boolean;
    data?: T[];
    items?: T[];
};

type RawProduct = {
    object: 'product';
    id: string;
    name: string;
    description?: string | null;
    kind?: string;
    list_price?: number;
    billing_cycle?: string;
    publication_name?: string | null;
    publication_description?: string | null;
    image_ids?: string[];
    category_id?: string | null;
    weight_grams?: number | null;
    created?: number;
};

type RawCategory = {
    object: 'product_category';
    id: string;
    name: string;
    slug?: string | null;
    parent_id?: string | null;
    sort_order?: number;
    image_url?: string | null;
};

type RawStock = {
    object: 'stock';
    id: string;
    product_id: string;
    quantity_on_hand: number;
    quantity_reserved: number;
    quantity_available: number;
    low_stock_threshold: number;
    track_inventory: boolean;
    created_at: string;
    updated_at: string;
    created?: number;
};

type RawCart = {
    object: 'cart';
    id: string;
    tenant_id: string;
    user_id?: string | null;
    session_id?: string | null;
    status: string;
    items?: RawCartItem[];
    expires_at?: string | null;
    created_at: string;
    updated_at: string;
    created?: number;
};

type RawCartItem = {
    object: 'cart_item';
    id: string;
    product_id: string;
    quantity: number;
    unit_price_nanodollar: number;
};

type RawCoupon = {
    object: 'coupon';
    id: string;
    tenant_id: string;
    code: string;
    discount_type: string;
    discount_value: number;
    currency: string;
    is_active: boolean;
    expires_at?: string | null;
    usage_limit?: number | null;
    used_count: number;
    minimum_order_amount?: number | null;
    use_per_user: boolean;
    discount_amount?: number | null;
    created_at?: string;
    updated_at?: string;
    created?: number;
};

type RawOrder = {
    object: 'order';
    id: string;
    tenant_id: string;
    cart_id?: string | null;
    user_id?: string | null;
    session_id?: string | null;
    status: string;
    fulfillment_method?: string | null;
    payment_method?: string | null;
    shipping_name?: string | null;
    shipping_address?: string | null;
    shipping_phone?: string | null;
    subtotal_nanodollar: number;
    discount_nanodollar: number;
    shipping_fee_nanodollar: number;
    total_nanodollar: number;
    items?: RawOrderItem[];
    confirmed_at?: string | null;
    shipped_at?: string | null;
    delivered_at?: string | null;
    cancelled_at?: string | null;
    pickup_requested_at?: string | null;
    pickup_deadline?: string | null;
    ready_at?: string | null;
    picked_up_at?: string | null;
    refunded_at?: string | null;
    payment_status?: string | null;
    checkout_url?: string | null;
    created_at: string;
    updated_at?: string;
    created?: number;
};

type RawOrderItem = {
    object: 'order_item';
    id: string;
    product_id: string;
    product_name: string;
    quantity: number;
    unit_price_nanodollar: number;
    subtotal_nanodollar: number;
};

type RawCustomer = {
    object: 'customer';
    id: string;
    name: string;
    email: string;
    phone?: string | null;
    description?: string | null;
};

function mapList<T, R>(
    response: RawList<T> | T[],
    mapper: (item: T) => R,
): StoreKitList<R> {
    const items = Array.isArray(response)
        ? response
        : (response.data ?? response.items ?? []);

    return {
        object: 'list',
        url: Array.isArray(response) ? '' : (response.url ?? ''),
        hasMore: Array.isArray(response)
            ? false
            : (response.has_more ?? response.hasMore ?? false),
        data: items.map(mapper),
    };
}

function mapProduct(raw: RawProduct): StoreKitProduct {
    return {
        object: 'product',
        id: raw.id,
        name: raw.name,
        description: raw.description,
        kind: raw.kind,
        listPrice: raw.list_price,
        billingCycle: raw.billing_cycle,
        publicationName: raw.publication_name,
        publicationDescription: raw.publication_description,
        imageIds: raw.image_ids ?? [],
        categoryId: raw.category_id,
        weightGrams: raw.weight_grams,
        created: raw.created,
    };
}

function mapCategory(raw: RawCategory): StoreKitCategory {
    return {
        object: 'product_category',
        id: raw.id,
        name: raw.name,
        slug: raw.slug,
        parentId: raw.parent_id,
        sortOrder: raw.sort_order,
        imageUrl: raw.image_url,
    };
}

function mapStock(raw: RawStock): StoreKitProductStock {
    return {
        object: 'stock',
        id: raw.id,
        productId: raw.product_id,
        quantityOnHand: raw.quantity_on_hand,
        quantityReserved: raw.quantity_reserved,
        quantityAvailable: raw.quantity_available,
        lowStockThreshold: raw.low_stock_threshold,
        trackInventory: raw.track_inventory,
        createdAt: raw.created_at,
        updatedAt: raw.updated_at,
        created: raw.created,
    };
}

function mapCart(raw: RawCart): StoreKitCart {
    return {
        object: 'cart',
        id: raw.id,
        tenantId: raw.tenant_id,
        userId: raw.user_id,
        sessionId: raw.session_id,
        status: raw.status,
        items: (raw.items ?? []).map(mapCartItem),
        expiresAt: raw.expires_at,
        createdAt: raw.created_at,
        updatedAt: raw.updated_at,
        created: raw.created,
    };
}

function mapCartItem(raw: RawCartItem): StoreKitCartItem {
    return {
        object: 'cart_item',
        id: raw.id,
        productId: raw.product_id,
        quantity: raw.quantity,
        unitPrice: money(raw.unit_price_nanodollar),
    };
}

function mapCoupon(raw: RawCoupon): StoreKitCoupon {
    return {
        object: 'coupon',
        id: raw.id,
        tenantId: raw.tenant_id,
        code: raw.code,
        discountType: raw.discount_type,
        discountValue: raw.discount_value,
        currency: raw.currency,
        isActive: raw.is_active,
        expiresAt: raw.expires_at,
        usageLimit: raw.usage_limit,
        usedCount: raw.used_count,
        minimumOrderAmount: raw.minimum_order_amount,
        usePerUser: raw.use_per_user,
        discountAmount: raw.discount_amount,
        createdAt: raw.created_at,
        updatedAt: raw.updated_at,
        created: raw.created,
    };
}

function mapOrder(raw: RawOrder): StoreKitOrder {
    return {
        object: 'order',
        id: raw.id,
        tenantId: raw.tenant_id,
        cartId: raw.cart_id,
        userId: raw.user_id,
        sessionId: raw.session_id,
        status: raw.status,
        fulfillmentMethod: raw.fulfillment_method,
        paymentMethod: raw.payment_method,
        shippingName: raw.shipping_name,
        shippingAddress: raw.shipping_address,
        shippingPhone: raw.shipping_phone,
        subtotal: money(raw.subtotal_nanodollar),
        discount: money(raw.discount_nanodollar),
        shippingFee: money(raw.shipping_fee_nanodollar),
        total: money(raw.total_nanodollar),
        items: (raw.items ?? []).map(mapOrderItem),
        confirmedAt: raw.confirmed_at,
        shippedAt: raw.shipped_at,
        deliveredAt: raw.delivered_at,
        cancelledAt: raw.cancelled_at,
        pickupRequestedAt: raw.pickup_requested_at,
        pickupDeadline: raw.pickup_deadline,
        readyAt: raw.ready_at,
        pickedUpAt: raw.picked_up_at,
        refundedAt: raw.refunded_at,
        paymentStatus: raw.payment_status,
        checkoutUrl: raw.checkout_url,
        createdAt: raw.created_at,
        updatedAt: raw.updated_at,
        created: raw.created,
    };
}

function mapOrderItem(raw: RawOrderItem): StoreKitOrderItem {
    return {
        object: 'order_item',
        id: raw.id,
        productId: raw.product_id,
        productName: raw.product_name,
        quantity: raw.quantity,
        unitPrice: money(raw.unit_price_nanodollar),
        subtotal: money(raw.subtotal_nanodollar),
    };
}

function mapCustomer(raw: RawCustomer): StoreKitCustomer {
    return {
        object: 'customer',
        id: raw.id,
        name: raw.name,
        email: raw.email,
        phone: raw.phone,
        description: raw.description,
    };
}

function mapAddressToRaw(address?: StoreKitAddress) {
    if (!address) {
        return undefined;
    }
    return {
        postal_code: address.postalCode,
        state: address.state,
        city: address.city,
        address1: address.address1,
        address2: address.address2,
    };
}

function money(nanodollar: number): StoreKitMoney {
    return {
        nanodollar,
        minorUnit: nanodollarToMinorUnit(nanodollar),
    };
}

function toQueryString(query?: Record<string, unknown>): string {
    if (!query) {
        return '';
    }

    return Object.keys(query)
        .filter((key) => query[key] !== undefined && query[key] !== null)
        .map((key) => {
            const value = query[key];
            return `${encodeURIComponent(key)}=${encodeURIComponent(
                String(value),
            )}`;
        })
        .join('&');
}

export * from './graphql';
