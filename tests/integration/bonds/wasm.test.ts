import { Order, TEST_ORDER, TEST_ORDER_ARRAY } from "wasm-utils"
import { Orderbook } from "../../../libraries/ts-bonds/src"

describe("wasm utils", async () => {
  before(async () => {})
  it("basic deserialization", async () => {
    const singleOrder: Order = TEST_ORDER()
    const multipleOrders: Order[] = TEST_ORDER_ARRAY()

    const multipleOrdersDeserialized = Orderbook.deserializeOrders(multipleOrders)

    // TODO: add checks
    // console.log(singleOrderDeserialized);
    // console.log(multipleOrdersDeserialized);
  })
})
