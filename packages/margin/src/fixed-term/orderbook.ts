import { FixedTermMarket } from "./fixedTerm"
import { get_orders_from_slab as getOrdersFromSlab, Order } from "@jet-lab/wasm"

export class Orderbook {
  readonly bids: Order[]
  readonly asks: Order[]

  private constructor(bids: Order[], asks: Order[]) {
    this.bids = bids
    this.asks = asks
  }

  static async load(fixedTermMarket: FixedTermMarket): Promise<Orderbook> {
    const asksBuf = (await fixedTermMarket.provider.connection.getAccountInfo(fixedTermMarket.info.asks))!.data
    const bidsBuf = (await fixedTermMarket.provider.connection.getAccountInfo(fixedTermMarket.info.bids))!.data

    const asks = getOrdersFromSlab(new Uint8Array(asksBuf))
    const bids = getOrdersFromSlab(new Uint8Array(bidsBuf))

    return new Orderbook(bids, asks)
  }
}
