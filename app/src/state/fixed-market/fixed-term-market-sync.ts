import { FixedTermMarket, JetMarket, JetMarketIdl, Orderbook } from '@jet-lab/fixed-market';
import { Program } from '@project-serum/anchor';
import { useEffect } from 'react';
import { atom, selector, useRecoilValue, useSetRecoilState } from 'recoil';
import { AirspaceConfig, FixedTermMarketConfig } from '@jet-lab/margin';
import { MainConfig } from '../config/marginConfig';
import { PublicKey } from '@solana/web3.js';
import { useProvider } from '@utils/jet/provider';
import { NetworkStateAtom } from '@state/network/network-state';
import { useLocation } from 'react-router-dom';

export const AllFixedTermMarketsAtom = atom<Array<MarketAndconfig>>({
  key: 'allFixedTermMarkets',
  default: [],
  dangerouslyAllowMutability: true
});

export const SelectedFixedTermMarketAtom = atom<number>({
  key: 'selectedFixedTermMarketIndex',
  default: 0,
  dangerouslyAllowMutability: true
});

export const FixedTermMarketAtom = selector<MarketAndconfig | null>({
  key: 'fixedTermMarketAtom',
  get: ({ get }) => {
    const list = get(AllFixedTermMarketsAtom);
    const selected = get(SelectedFixedTermMarketAtom);
    return list[selected];
  },
  dangerouslyAllowMutability: true
});

export type CurrentOrderTab = 'borrow-now' | 'lend-now' | 'offer-loan' | 'request-loan';

export const CurrentOrderTabAtom = atom<CurrentOrderTab>({
  key: 'current-fixed-term-order-tab',
  default: null
});

export interface ExtendedOrderBook extends Orderbook {
  name: string;
}

export const AllFixedTermMarketsOrderBooksAtom = selector<ExtendedOrderBook[]>({
  key: 'allFixedTermMarketOrderBooks',
  get: async ({ get }) => {
    const list = get(AllFixedTermMarketsAtom);
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
  market: FixedTermMarket;
  config: FixedTermMarketConfig;
  name: string;
}
export const useFixedTermSync = (): void => {
  const { provider } = useProvider();
  const setMarkets = useSetRecoilState(AllFixedTermMarketsAtom);
  const config = useRecoilValue(MainConfig);
  const networkState = useRecoilValue(NetworkStateAtom);
  const setCurrentOrderTab = useSetRecoilState(CurrentOrderTabAtom);
  const { pathname } = useLocation();

  const loadFixedTermMarkets = async (
    airspace: AirspaceConfig,
    program: Program<JetMarket>,
    marginProgramId: PublicKey
  ) => {
    const markets: MarketAndconfig[] = await Promise.all(
      Object.entries(airspace.fixedTermMarkets).map(async ([name, marketConfig]) => {
        const market = await FixedTermMarket.load(program, marketConfig.market, marginProgramId);
        return { market, config: marketConfig, name };
      })
    );
    setMarkets(markets);
  };

  useEffect(() => {
    if (networkState === 'connected' && config?.fixedTermMarketProgramId) {
      const program = new Program(JetMarketIdl, config.fixedTermMarketProgramId, provider);
      const airspace = config.airspaces.find(airspace => airspace.name === 'default');
      loadFixedTermMarkets(airspace, program, new PublicKey(config.marginProgramId));
    }
  }, [config, networkState]);

  useEffect(() => {
    if (pathname.includes('/fixed-lend')) {
      setCurrentOrderTab('offer-loan');
    } else if (pathname.includes('/fixed-borrow')) {
      setCurrentOrderTab('request-loan');
    }
  }, [pathname]);

  return null;
};
