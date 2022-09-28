import { BondMarket, JetBondsIdl, Orderbook } from "@jet-lab/margin"
import { Program } from "@project-serum/anchor"
import { useEffect } from "react"
import { atom, selector, useRecoilState } from "recoil"
import { useProvider } from "../../utils/jet/provider"
import { generateMarkets, generateOrderBook, MockBook, MockMarket } from "./mocks"

// TODO, Eventually this should be an atom family
export const FixedMarketAtom = atom<BondMarket | null>({
  key: "fixedMarketAtom",
  default: null,
  dangerouslyAllowMutability: true
})

export const FixedMarketOrderBookAtom = selector<Orderbook>({
  key: "fixedMarketOrderBookAtom",
  get: async ({ get }) => {
    const market = get(FixedMarketAtom)
    if (market) {
      const rawOrderBook = await market.fetchOrderbook()
      return {
        asks: rawOrderBook.asks.sort((a, b) => Number(a.limit_price) - Number(b.limit_price)),
        bids: rawOrderBook.bids.sort((a, b) => Number(b.limit_price) - Number(a.limit_price))
      }
    } else {
      return {
        asks: [],
        bids: []
      }
    }
  }
})

export const useFixedTermSync = () => {
  const { provider } = useProvider()
  const [market, setMarket] = useRecoilState(FixedMarketAtom)
  useEffect(() => {
    const program = new Program(JetBondsIdl, "DMCynpScPPEFj6h5zbVrdMTd1HoBWmLyRhzbTfTYyN1Q", provider)
    BondMarket.load(program, "HWg6LPw2sjTBfBeu8Au3dHcsnsSRCmnkaoPqZBeqS7bt").then(result => {
      if (!market || !result.address.equals(market.address)) {
        setMarket(result)
      }
    })
  }, [provider])
  return null
}

// Mocked Fixed Markets State
export const AllFixedMarketsAtom = atom<MockMarket[]>({
  key: "allFixedMarkets",
  default: generateMarkets()
})

export const SelectedFixedMarketAtom = atom<number>({
  key: "selectedFixedMarketIndex",
  default: 0
})

export const AllFixedMarketsOrderBooksAtom = selector<MockBook[]>({
  key: "allFixedMarketOrderBooks",
  get: ({ get }) => {
    const list = get(AllFixedMarketsAtom)
    return list.map(market => generateOrderBook(market))
  }
})
