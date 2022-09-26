import { get_orders_from_slab as getOrdersFromSlab } from "../wasm-utils/pkg";
export class Orderbook {
    constructor(bids, asks) {
        this.bids = bids;
        this.asks = asks;
    }
    static async load(bondMarket) {
        const asksBuf = (await bondMarket.provider.connection.getAccountInfo(bondMarket.info.asks)).data;
        const bidsBuf = (await bondMarket.provider.connection.getAccountInfo(bondMarket.info.bids)).data;
        const asksRaw = getOrdersFromSlab(new Uint8Array(asksBuf));
        const bidsRaw = getOrdersFromSlab(new Uint8Array(bidsBuf));
        const asks = this.deserializeOrders(asksRaw);
        const bids = this.deserializeOrders(bidsRaw);
        return new Orderbook(bids, asks);
    }
    static deserializeOrders(rawOrders) {
        let orders = [];
        for (let order in rawOrders) {
            orders.push(rawOrders[order]);
        }
        return orders;
    }
}
//# sourceMappingURL=orderbook.js.map