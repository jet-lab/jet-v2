"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Orderbook = void 0;
const pkg_1 = require("../wasm-utils/pkg");
class Orderbook {
    constructor(bids, asks) {
        this.bids = bids;
        this.asks = asks;
    }
    static async load(bondMarket) {
        const asksBuf = (await bondMarket.provider.connection.getAccountInfo(bondMarket.info.asks)).data;
        const bidsBuf = (await bondMarket.provider.connection.getAccountInfo(bondMarket.info.bids)).data;
        const asksRaw = (0, pkg_1.get_orders_from_slab)(new Uint8Array(asksBuf));
        const bidsRaw = (0, pkg_1.get_orders_from_slab)(new Uint8Array(bidsBuf));
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
exports.Orderbook = Orderbook;
//# sourceMappingURL=orderbook.js.map