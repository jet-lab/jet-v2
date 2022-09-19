import { BondMarket, JetBondsIdl, Orderbook } from '@jet-lab/jet-bonds-client';
import { Program } from '@project-serum/anchor';
import { useEffect } from 'react';
import { atom, selector, useRecoilState } from 'recoil';
import { useProvider } from '../../utils/jet/provider';

// TODO, Eventually this should be an atom family
export const FixedMarketAtom = atom<BondMarket | null>({
  key: 'fixedMarketAtom',
  default: null,
  dangerouslyAllowMutability: true
});

export const FixedMarketOrderBookAtom = selector<Orderbook>({
  key: 'fixedMarketOrderBookAtom',
  get: async ({ get }) => {
    const market = get(FixedMarketAtom);
    if (market) {
      const rawOrderBook = await market.fetchOrderbook();
      return {
        asks: rawOrderBook.asks.sort((a, b) => Number(a.price) - Number(b.price)),
        bids: rawOrderBook.bids.sort((a, b) => Number(b.price) - Number(a.price))
      };
    } else {
      return {
        asks: [],
        bids: []
      };
    }
  }
});

export const useFixedTermSync = () => {
  const { provider } = useProvider();
  const [market, setMarket] = useRecoilState(FixedMarketAtom);
  useEffect(() => {
    const program = new Program(JetBondsIdl, 'DMCynpScPPEFj6h5zbVrdMTd1HoBWmLyRhzbTfTYyN1Q', provider);
    BondMarket.load(program, 'HWg6LPw2sjTBfBeu8Au3dHcsnsSRCmnkaoPqZBeqS7bt').then(result => {
      if (!market || !result.address.equals(market.address)) {
        setMarket(result);
      }
    });
  }, [provider]);
  return null;
};
