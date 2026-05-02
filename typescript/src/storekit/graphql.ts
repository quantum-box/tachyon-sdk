import type {
    AddCartItemParams,
    CheckoutParams,
    CreateCartParams,
    ListOrdersParams,
    ListProductsParams,
    RemoveCartItemParams,
    StoreKitCart,
    StoreKitCategory,
    StoreKitCoupon,
    StoreKitList,
    StoreKitMoney,
    StoreKitOrder,
    StoreKitProduct,
    StoreKitProductStock,
    UpdateCartItemParams,
    ValidateCouponParams,
} from './index';
import { nanodollarToMinorUnit } from './index';

export type StoreKitGraphqlSdk = {
    getShopProducts(variables?: {
        categoryId?: string | null;
        search?: string | null;
        inStock?: boolean | null;
        limit?: number | null;
        offset?: number | null;
    }): Promise<{
        storefrontProducts: {
            limit: number;
            offset: number;
            items: StoreKitGraphqlProduct[];
        };
        storefrontCategories: StoreKitGraphqlCategory[];
    }>;
    getShopProductDetail(variables: { productId: string }): Promise<{
        storefrontProduct: StoreKitGraphqlProduct;
        productStock: {
            quantityAvailable: number;
            trackInventory: boolean;
        };
    }>;
    createCartForShop(variables: {
        input: { userId?: string | null; sessionId?: string | null };
    }): Promise<{ createCart: StoreKitGraphqlCart }>;
    addCartItemForShop(variables: {
        cartId: string;
        input: { productId: string; quantity: number };
    }): Promise<{ addCartItem: StoreKitGraphqlCart }>;
    updateCartItemForShop(variables: {
        cartId: string;
        itemId: string;
        input: { quantity: number };
    }): Promise<{ updateCartItem: StoreKitGraphqlCart }>;
    removeCartItemForShop(variables: {
        cartId: string;
        itemId: string;
    }): Promise<{ removeCartItem: boolean }>;
    clearCartForShop(variables: {
        cartId: string;
    }): Promise<{ clearCart: boolean }>;
    getCartForShop(variables: {
        cartId: string;
    }): Promise<{ cart: StoreKitGraphqlCart }>;
    checkoutForShop(variables: {
        input: StoreKitGraphqlCheckoutInput;
    }): Promise<{ checkout: StoreKitGraphqlOrder }>;
    selectPickupDatetimeForShop(variables: {
        orderId: string;
        pickupRequestedAt?: string | null;
    }): Promise<{ selectPickupDatetime: StoreKitGraphqlOrder }>;
    validateCouponForShop(variables: {
        code: string;
        subtotalNanodollar?: number | null;
    }): Promise<{ validateCoupon: StoreKitGraphqlCoupon }>;
    getConsumerOrdersForShop(variables?: {
        userId?: string | null;
        sessionId?: string | null;
        limit?: number | null;
        offset?: number | null;
    }): Promise<{
        consumerOrders: {
            limit: number;
            offset: number;
            items: StoreKitGraphqlOrder[];
        };
    }>;
    getConsumerOrderDetailForShop(variables: {
        orderId: string;
    }): Promise<{ consumerOrder: StoreKitGraphqlOrder }>;
};

export type StoreKitGraphqlClientOptions = {
    sdk: StoreKitGraphqlSdk;
    sessionId?: string;
    userId?: string;
};

export function createStoreKitClientFromGraphqlSdk(
    options: StoreKitGraphqlClientOptions,
): StoreKitGraphqlClient {
    return new StoreKitGraphqlClient(options);
}

export class StoreKitGraphqlClient {
    readonly products = new StoreKitGraphqlProductsResource(this);
    readonly cart = new StoreKitGraphqlCartResource(this);
    readonly coupons = new StoreKitGraphqlCouponsResource(this);
    readonly checkout = new StoreKitGraphqlCheckoutResource(this);
    readonly orders = new StoreKitGraphqlOrdersResource(this);

    constructor(readonly options: StoreKitGraphqlClientOptions) {}

    defaultSessionId(): string | undefined {
        return this.options.sessionId;
    }

    defaultUserId(): string | undefined {
        return this.options.userId;
    }
}

export class StoreKitGraphqlProductsResource {
    constructor(private readonly client: StoreKitGraphqlClient) {}

    async list(
        params: ListProductsParams = {},
    ): Promise<StoreKitList<StoreKitProduct>> {
        const response = await this.client.options.sdk.getShopProducts({
            categoryId: params.categoryId,
            search: params.search,
            limit: params.limit,
            offset: offsetFromCursor(params.startingAfter),
        });
        return {
            object: 'list',
            url: 'graphql:getShopProducts',
            hasMore:
                response.storefrontProducts.items.length >=
                response.storefrontProducts.limit,
            data: response.storefrontProducts.items.map(mapProduct),
        };
    }

    async get(
        productId: string,
    ): Promise<StoreKitProduct & { stock: StoreKitProductStock }> {
        const response =
            await this.client.options.sdk.getShopProductDetail({ productId });
        return {
            ...mapProduct(response.storefrontProduct),
            stock: {
                object: 'stock',
                id: response.storefrontProduct.id,
                productId: response.storefrontProduct.id,
                quantityOnHand: response.productStock.quantityAvailable,
                quantityReserved: 0,
                quantityAvailable: response.productStock.quantityAvailable,
                lowStockThreshold: 0,
                trackInventory: response.productStock.trackInventory,
                createdAt: '',
                updatedAt: '',
            },
        };
    }

    async listCategories(): Promise<StoreKitList<StoreKitCategory>> {
        const response = await this.client.options.sdk.getShopProducts({
            limit: 1,
            offset: 0,
        });
        return {
            object: 'list',
            url: 'graphql:storefrontCategories',
            hasMore: false,
            data: response.storefrontCategories.map(mapCategory),
        };
    }
}

export class StoreKitGraphqlCartResource {
    constructor(private readonly client: StoreKitGraphqlClient) {}

    async create(params: CreateCartParams = {}): Promise<StoreKitCart> {
        const response = await this.client.options.sdk.createCartForShop({
            input: {
                userId: params.userId ?? this.client.defaultUserId(),
                sessionId: params.sessionId ?? this.client.defaultSessionId(),
            },
        });
        return mapCart(response.createCart);
    }

    async get(cartId: string): Promise<StoreKitCart> {
        const response = await this.client.options.sdk.getCartForShop({
            cartId,
        });
        return mapCart(response.cart);
    }

    async addItem(params: AddCartItemParams): Promise<StoreKitCart> {
        const response = await this.client.options.sdk.addCartItemForShop({
            cartId: params.cartId,
            input: {
                productId: params.productId,
                quantity: params.quantity,
            },
        });
        return mapCart(response.addCartItem);
    }

    async updateItem(params: UpdateCartItemParams): Promise<StoreKitCart> {
        const response = await this.client.options.sdk.updateCartItemForShop({
            cartId: params.cartId,
            itemId: params.itemId,
            input: { quantity: params.quantity },
        });
        return mapCart(response.updateCartItem);
    }

    async removeItem(params: RemoveCartItemParams): Promise<{ ok: boolean }> {
        const response = await this.client.options.sdk.removeCartItemForShop({
            cartId: params.cartId,
            itemId: params.itemId,
        });
        return { ok: response.removeCartItem };
    }

    async clear(cartId: string): Promise<{ ok: boolean }> {
        const response = await this.client.options.sdk.clearCartForShop({
            cartId,
        });
        return { ok: response.clearCart };
    }
}

export class StoreKitGraphqlCouponsResource {
    constructor(private readonly client: StoreKitGraphqlClient) {}

    async validate(params: ValidateCouponParams): Promise<StoreKitCoupon> {
        const response = await this.client.options.sdk.validateCouponForShop({
            code: params.code,
            subtotalNanodollar: params.subtotalNanodollar,
        });
        return {
            object: 'coupon',
            id: response.validateCoupon.id,
            tenantId: '',
            code: response.validateCoupon.code,
            discountType: response.validateCoupon.discountType,
            discountValue: response.validateCoupon.discountValue,
            currency: response.validateCoupon.currency,
            isActive: response.validateCoupon.isActive,
            usedCount: 0,
            usePerUser: false,
            discountAmount: response.validateCoupon.discountAmount,
        };
    }
}

export class StoreKitGraphqlCheckoutResource {
    constructor(private readonly client: StoreKitGraphqlClient) {}

    async create(params: CheckoutParams): Promise<StoreKitOrder> {
        const response = await this.client.options.sdk.checkoutForShop({
            input: {
                cartId: params.cartId,
                shippingName: params.shippingName,
                shippingAddress: params.shippingAddress,
                shippingPhone: params.shippingPhone,
                customerEmail: params.customerEmail,
                storeId: params.storeId,
                pickupRequestedAt: params.pickupRequestedAt,
                fulfillmentMethod: params.fulfillmentMethod,
                paymentMethod: params.paymentMethod,
                couponCode: params.couponCode,
                successUrl: params.successUrl,
                cancelUrl: params.cancelUrl,
            },
        });
        return mapOrder(response.checkout);
    }

    async confirm(_orderId: string): Promise<StoreKitOrder> {
        throw new Error(
            'checkout.confirm is not exposed by the bakuure-api GraphQL shop schema yet',
        );
    }
}

export class StoreKitGraphqlOrdersResource {
    constructor(private readonly client: StoreKitGraphqlClient) {}

    async list(
        params: ListOrdersParams = {},
    ): Promise<StoreKitList<StoreKitOrder>> {
        const response =
            await this.client.options.sdk.getConsumerOrdersForShop({
                userId: params.userId ?? this.client.defaultUserId(),
                sessionId: params.sessionId ?? this.client.defaultSessionId(),
                limit: params.limit,
                offset: offsetFromCursor(params.startingAfter),
            });
        return {
            object: 'list',
            url: 'graphql:getConsumerOrdersForShop',
            hasMore:
                response.consumerOrders.items.length >=
                response.consumerOrders.limit,
            data: response.consumerOrders.items.map(mapOrder),
        };
    }

    async get(orderId: string): Promise<StoreKitOrder> {
        const response =
            await this.client.options.sdk.getConsumerOrderDetailForShop({
                orderId,
            });
        return mapOrder(response.consumerOrder);
    }

    async selectPickupDatetime(params: {
        orderId: string;
        pickupRequestedAt?: string | null;
    }): Promise<StoreKitOrder> {
        const response =
            await this.client.options.sdk.selectPickupDatetimeForShop(params);
        return mapOrder(response.selectPickupDatetime);
    }
}

type StoreKitGraphqlProduct = {
    id: string;
    name: string;
    description?: string | null;
    kind?: string;
    listPrice: number;
    billingCycle?: string;
    publicationName?: string | null;
    publicationDescription?: string | null;
    imageIds: string[];
    categoryId?: string | null;
    weightGrams?: number | null;
};

type StoreKitGraphqlCategory = {
    id: string;
    name: string;
    slug: string;
};

type StoreKitGraphqlCart = {
    id: string;
    status: string;
    createdAt?: string;
    updatedAt?: string;
    items: StoreKitGraphqlCartItem[];
};

type StoreKitGraphqlCartItem = {
    id: string;
    productId: string;
    quantity: number;
    unitPriceNanodollar: string | number;
};

type StoreKitGraphqlOrder = {
    id: string;
    status: string;
    totalNanodollar?: string | number;
    subtotalNanodollar?: string | number;
    discountNanodollar?: string | number;
    shippingFeeNanodollar?: string | number;
    fulfillmentMethod?: string | null;
    paymentMethod?: string | null;
    shippingName?: string | null;
    shippingAddress?: string | null;
    shippingPhone?: string | null;
    pickupRequestedAt?: string | null;
    pickupDeadline?: string | null;
    checkoutUrl?: string | null;
    confirmedAt?: string | null;
    shippedAt?: string | null;
    deliveredAt?: string | null;
    cancelledAt?: string | null;
    createdAt?: string;
    items?: StoreKitGraphqlOrderItem[];
};

type StoreKitGraphqlOrderItem = {
    id: string;
    productId?: string;
    productName: string;
    quantity: number;
    unitPriceNanodollar?: string | number;
    subtotalNanodollar?: string | number;
};

type StoreKitGraphqlCoupon = {
    id: string;
    code: string;
    discountType: string;
    discountValue: number;
    currency: string;
    isActive: boolean;
    discountAmount?: number | null;
};

type StoreKitGraphqlCheckoutInput = {
    cartId: string;
    shippingName?: string;
    shippingAddress?: string;
    shippingPhone?: string;
    customerEmail?: string;
    storeId?: string;
    pickupRequestedAt?: string;
    fulfillmentMethod?: string;
    paymentMethod?: string;
    couponCode?: string;
    successUrl?: string;
    cancelUrl?: string;
};

function mapProduct(product: StoreKitGraphqlProduct): StoreKitProduct {
    return {
        object: 'product',
        id: product.id,
        name: product.name,
        description: product.description,
        kind: product.kind,
        listPrice: product.listPrice,
        billingCycle: product.billingCycle,
        publicationName: product.publicationName,
        publicationDescription: product.publicationDescription,
        imageIds: product.imageIds,
        categoryId: product.categoryId,
        weightGrams: product.weightGrams,
    };
}

function mapCategory(category: StoreKitGraphqlCategory): StoreKitCategory {
    return {
        object: 'product_category',
        id: category.id,
        name: category.name,
        slug: category.slug,
    };
}

function mapCart(cart: StoreKitGraphqlCart): StoreKitCart {
    return {
        object: 'cart',
        id: cart.id,
        tenantId: '',
        status: cart.status,
        items: cart.items.map((item) => ({
            object: 'cart_item',
            id: item.id,
            productId: item.productId,
            quantity: item.quantity,
            unitPrice: money(item.unitPriceNanodollar),
        })),
        createdAt: cart.createdAt ?? '',
        updatedAt: cart.updatedAt ?? '',
    };
}

function mapOrder(order: StoreKitGraphqlOrder): StoreKitOrder {
    return {
        object: 'order',
        id: order.id,
        tenantId: '',
        status: order.status,
        fulfillmentMethod: order.fulfillmentMethod,
        paymentMethod: order.paymentMethod,
        shippingName: order.shippingName,
        shippingAddress: order.shippingAddress,
        shippingPhone: order.shippingPhone,
        subtotal: money(order.subtotalNanodollar ?? 0),
        discount: money(order.discountNanodollar ?? 0),
        shippingFee: money(order.shippingFeeNanodollar ?? 0),
        total: money(order.totalNanodollar ?? 0),
        items: (order.items ?? []).map((item) => ({
            object: 'order_item',
            id: item.id,
            productId: item.productId ?? '',
            productName: item.productName,
            quantity: item.quantity,
            unitPrice: money(item.unitPriceNanodollar ?? 0),
            subtotal: money(item.subtotalNanodollar ?? 0),
        })),
        pickupRequestedAt: order.pickupRequestedAt,
        pickupDeadline: order.pickupDeadline,
        checkoutUrl: order.checkoutUrl,
        confirmedAt: order.confirmedAt,
        shippedAt: order.shippedAt,
        deliveredAt: order.deliveredAt,
        cancelledAt: order.cancelledAt,
        createdAt: order.createdAt ?? '',
    };
}

function money(value: string | number): StoreKitMoney {
    const nanodollar = Number(value);
    return {
        nanodollar,
        minorUnit: nanodollarToMinorUnit(nanodollar),
    };
}

function offsetFromCursor(cursor?: string): number | undefined {
    if (!cursor) {
        return undefined;
    }
    const parsed = Number(cursor);
    return Number.isFinite(parsed) ? parsed : undefined;
}
