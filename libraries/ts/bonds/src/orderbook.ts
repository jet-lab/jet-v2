import { FixedMarket } from "./fixedMarket"
import { get_orders_from_slab as getOrdersFromSlab, Order } from "./wasm-utils"

export class Orderbook {
  readonly bids: Order[]
  readonly asks: Order[]

  private constructor(bids: Order[], asks: Order[]) {
    this.bids = bids
    this.asks = asks
  }

  static async load(fixedMarket: FixedMarket): Promise<Orderbook> {
    const asksBuf = (await fixedMarket.provider.connection.getAccountInfo(fixedMarket.info.asks))!.data
    const bidsBuf = (await fixedMarket.provider.connection.getAccountInfo(fixedMarket.info.bids))!.data

    const asks = getOrdersFromSlab(new Uint8Array(asksBuf))
    const bids = getOrdersFromSlab(new Uint8Array(bidsBuf))

    return new Orderbook(bids, asks)
  }
}
