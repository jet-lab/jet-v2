import { BondMarket } from "./bondMarket";
import { Order } from "../wasm-utils/pkg";
export declare class Orderbook {
    readonly bids: Order[];
    readonly asks: Order[];
    private constructor();
    static load(bondMarket: BondMarket): Promise<Orderbook>;
    static deserializeOrders(rawOrders: any[]): Order[];
}
//# sourceMappingURL=orderbook.d.ts.map