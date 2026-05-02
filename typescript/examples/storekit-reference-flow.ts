import { createStoreKitClient, StoreKitError } from '../src/storekit';

async function main() {
    const storekit = createStoreKitClient({
        baseUrl:
            process.env.BAKUURE_API_BASE_URL ??
            'http://localhost:14001',
        operatorId:
            process.env.BAKUURE_OPERATOR_ID ??
            'tn_01hjryxysgey07h5jz5wagqj0m',
        publicApiKey: process.env.BAKUURE_API_KEY ?? 'dummy-token',
        sessionId:
            process.env.STOREKIT_SESSION_ID ??
            `storekit-example-${Date.now()}`,
    });

    const products = await storekit.products.list({ limit: 20 });
    const product = products.data[0];

    if (!product) {
        throw new Error('No StoreKit products are available.');
    }

    const stock = await storekit.products.getStock(product.id);
    const cart = await storekit.cart.create();
    const cartWithItem = await storekit.cart.addItem({
        cartId: cart.id,
        productId: product.id,
        quantity: 1,
    });

    const order = await storekit.checkout.create({
        cartId: cartWithItem.id,
        fulfillmentMethod: 'delivery',
        paymentMethod: 'cash_on_pickup',
    });

    const fetchedOrder = await storekit.orders.get(order.id);

    console.log({
        product: product.name,
        stock: stock.quantityAvailable,
        cartId: cartWithItem.id,
        orderId: fetchedOrder.id,
        total: fetchedOrder.total.nanodollar,
    });
}

main().catch((error) => {
    if (error instanceof StoreKitError) {
        console.error({
            status: error.status,
            type: error.type,
            code: error.code,
            message: error.message,
        });
        process.exit(1);
    }

    throw error;
});
