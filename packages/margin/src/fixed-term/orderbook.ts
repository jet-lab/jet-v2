import { FixedTermMarket } from "./fixedTerm"
import { get_orders_from_slab as getOrdersFromSlab, Order } from "../wasm"

export class Orderbook {
  readonly bids: Order[]
  readonly asks: Order[]

  private constructor(bids: Order[], asks: Order[]) {
    this.bids = bids
    this.asks = asks
  }

  static async load(fixedTermMarket: FixedTermMarket): Promise<Orderbook> {
    let asks: Order[] = []
    let bids: Order[] = []
    try {
      const asksBuf = (await fixedTermMarket.provider.connection.getAccountInfo(fixedTermMarket.info.asks))!.data
      asks = getOrdersFromSlab(new Uint8Array(asksBuf))
    } catch (e) {
      console.log(e)
    }
    try {
      const bidsBuf = (await fixedTermMarket.provider.connection.getAccountInfo(fixedTermMarket.info.bids))!.data
      bids = getOrdersFromSlab(new Uint8Array(bidsBuf))
    } catch (e) {
      console.log(e)
    }
    return new Orderbook(bids, asks)
  }
}
