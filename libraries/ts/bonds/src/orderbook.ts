import { BondMarket } from "./bondMarket"
import { get_orders_from_slab as getOrdersFromSlab, Order } from "./wasm-utils"

export class Orderbook {
  readonly bids: Order[]
  readonly asks: Order[]

  private constructor(bids: Order[], asks: Order[]) {
    this.bids = bids
    this.asks = asks
  }

  static async load(bondMarket: BondMarket): Promise<Orderbook> {
    const asksBuf = (await bondMarket.provider.connection.getAccountInfo(bondMarket.info.asks))!.data
    const bidsBuf = (await bondMarket.provider.connection.getAccountInfo(bondMarket.info.bids))!.data

    const asks = getOrdersFromSlab(new Uint8Array(asksBuf))
    const bids = getOrdersFromSlab(new Uint8Array(bidsBuf))

    return new Orderbook(bids, asks)
  }
}
