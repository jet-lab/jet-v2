import { BondMarket, JetBonds, JetBondsIdl, Orderbook } from '@jet-lab/jet-bonds-client';
import { Program } from '@project-serum/anchor';
import { useEffect } from 'react';
import { atom, selector, useRecoilValue, useSetRecoilState } from 'recoil';
import { AirspaceConfig, BondMarketConfig } from '@jet-lab/margin';
import { MainConfig } from '../config/marginConfig';
import { PublicKey } from '@solana/web3.js';
import { useProvider } from '@utils/jet/provider';
import { NetworkStateAtom } from '@state/network/network-state';
import { useLocation } from 'react-router-dom';

export const AllFixedMarketsAtom = atom<Array<MarketAndconfig>>({
  key: 'allFixedMarkets',
  default: [],
  dangerouslyAllowMutability: true
});

export const SelectedFixedMarketAtom = atom<number>({
  key: 'selectedFixedMarketIndex',
  default: 0,
  dangerouslyAllowMutability: true
});

export const FixedMarketAtom = selector<MarketAndconfig | null>({
  key: 'fixedMarketAtom',
  get: ({ get }) => {
    const list = get(AllFixedMarketsAtom);
    const selected = get(SelectedFixedMarketAtom);
    return list[selected];
  },
  dangerouslyAllowMutability: true
});

export type CurrentOrderTab = 'borrow-now' | 'lend-now' | 'offer-loan' | 'request-loan' | 'not_set';

export const CurrentOrderTabAtom = atom<CurrentOrderTab>({
  key: 'current-fixed-term-order-tab',
  default: 'not_set'
});

export interface ExtendedOrderBook extends Orderbook {
  name: string;
}

export const AllFixedMarketsOrderBooksAtom = selector<ExtendedOrderBook[]>({
  key: 'allFixedMarketOrderBooks',
  get: async ({ get }) => {
    const list = get(AllFixedMarketsAtom);
    return await Promise.all(
      list.map(async market => {
        const raw = await market.market.fetchOrderbook();
        return {
          name: market.name,
          asks: raw.asks.sort((a, b) => Number(a.limit_price) - Number(b.limit_price)),
          bids: raw.bids.sort((a, b) => Number(b.limit_price) - Number(a.limit_price))
        };
      })
    );
  }
});

export interface MarketAndconfig {
  market: BondMarket;
  config: BondMarketConfig;
  name: string;
}
export const useFixedTermSync = (): void => {
  const { provider } = useProvider();
  const setMarkets = useSetRecoilState(AllFixedMarketsAtom);
  const config = useRecoilValue(MainConfig);
  const networkState = useRecoilValue(NetworkStateAtom);
  const setCurrentOrderTab = useSetRecoilState(CurrentOrderTabAtom);
  const { pathname } = useLocation();

  const loadBondMarkets = async (airspace: AirspaceConfig, program: Program<JetBonds>, marginProgramId: PublicKey) => {
    const markets: MarketAndconfig[] = await Promise.all(
      Object.entries(airspace.bondMarkets).map(async ([name, marketConfig]) => {
        const market = await BondMarket.load(program, marketConfig.bondManager, marginProgramId);
        return { market, config: marketConfig, name };
      })
    );
    setMarkets(markets);
  };

  useEffect(() => {
    if (networkState === 'connected' && config?.bondsProgramId) {
      const program = new Program(JetBondsIdl, config.bondsProgramId, provider);
      const airspace = config.airspaces.find(airspace => airspace.name === 'default');
      if (airspace) {
        loadBondMarkets(airspace, program, new PublicKey(config.marginProgramId));
      }
    }
  }, [config, networkState]);

  useEffect(() => {
    if (pathname.includes('/fixed-lend')) {
      setCurrentOrderTab('offer-loan');
    } else if (pathname.includes('/fixed-borrow')) {
      setCurrentOrderTab('request-loan');
    }
  }, [pathname]);

  return;
};
