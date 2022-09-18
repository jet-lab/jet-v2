import { BondMarket } from "./bondMarket";
import {
  get_orders_from_slab as fetchOrdersFromSlab,
  Order,
} from "../wasm-utils/pkg";

export class Orderbook {
  readonly bids: Order[];
  readonly asks: Order[];

  private constructor(bids: Order[], asks: Order[]) {
    this.bids = bids;
    this.asks = asks;
  }

  static async load(bondMarket: BondMarket): Promise<Orderbook> {
    const asksKey = bondMarket.info.asks;
    const bidsKey = bondMarket.info.bids;
    const asksBuf = (await bondMarket.provider.connection.getAccountInfo(
      asksKey
    ))!.data;
    const bidsBuf = (await bondMarket.provider.connection.getAccountInfo(
      bidsKey
    ))!.data;

    const asksRaw = fetchOrdersFromSlab(new Uint8Array(asksBuf));
    const bidsRaw = fetchOrdersFromSlab(new Uint8Array(bidsBuf));

    const asks = this.deserializeOrders(asksRaw);
    const bids = this.deserializeOrders(bidsRaw);

    return new Orderbook(bids, asks);
  }

  static deserializeOrders(rawOrders: any[]): Order[] {
    let orders: Order[] = [];
    for (let order in rawOrders) {
      orders.push(rawOrders[order]);
    }
    return orders;
  }
}
