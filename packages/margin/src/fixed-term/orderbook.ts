import { FixedTermMarket } from "./fixedTerm"
import { OrderbookModel } from "../wasm"
import { bnToBigInt } from "../token";

export class Orderbook {
  model: OrderbookModel

  private constructor(model: OrderbookModel) {
    this.model = model;
  }

  static async load(fixedTermMarket: FixedTermMarket): Promise<Orderbook> {
    const asksBuf = (await fixedTermMarket.provider.connection.getAccountInfo(fixedTermMarket.info.asks))!.data
    const bidsBuf = (await fixedTermMarket.provider.connection.getAccountInfo(fixedTermMarket.info.bids))!.data

    console.log("FIXME: tenor is broken")
    const model = new OrderbookModel(bnToBigInt(fixedTermMarket.info.borrowTenor));
    model.refresh(bidsBuf, asksBuf);
    const orderbook = new Orderbook(model);

    return orderbook;
  }

  public liquidityChartData(side: string): any {
    const sample = this.model.sampleLiquidity(side);

    return sample;
  }
}
