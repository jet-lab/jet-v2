import { OrderAmount } from "wasm-utils";
import { TEST_ORDER_AMOUNT } from "wasm-utils";
import { Order, TEST_ORDER, TEST_ORDER_ARRAY } from "wasm-utils";

describe("wasm utils", async () => {
  before(async () => {});
  it("basic deserialization", async () => {
    const singleOrder: Order = TEST_ORDER();
    const multipleOrders: Order[] = TEST_ORDER_ARRAY();
    const orderAmount: OrderAmount = TEST_ORDER_AMOUNT();

    // TODO asserts
  });
});
