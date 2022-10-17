import { BondMarket, JetBonds, JetBondsIdl, Orderbook } from '@jet-lab/jet-bonds-client';
import { Program } from '@project-serum/anchor';
import { useEffect } from 'react';
import { atom, selector, useRecoilValue, useSetRecoilState } from 'recoil';
import { useProvider } from '../../utils/jet/provider';
import { AirspaceConfig, BondMarketConfig } from '@jet-lab/margin';
import { MainConfig } from '../config/marginConfig';

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

export const AllFixedMarketsOrderBooksAtom = selector<Orderbook[]>({
  key: 'allFixedMarketOrderBooks',
  get: async ({ get }) => {
    const list = get(AllFixedMarketsAtom);
    return Promise.all(
      list.map(async market => {
        const raw = await market.market.fetchOrderbook();
        return {
          asks: raw.asks.sort((a, b) => Number(a.limit_price) - Number(b.limit_price)),
          bids: raw.bids.sort((a, b) => Number(b.limit_price) - Number(a.limit_price))
        };
      })
    );
  },
  dangerouslyAllowMutability: true
});

interface MarketAndconfig {
  market: BondMarket;
  config: BondMarketConfig;
  name: string;
}
export const useFixedTermSync = (): void => {
  const { provider } = useProvider();
  const setMarkets = useSetRecoilState(AllFixedMarketsAtom);
  const config = useRecoilValue(MainConfig);

  const loadBondMarkets = async (airspace: AirspaceConfig, program: Program<JetBonds>, bondsProgramId: string) => {
    const markets: MarketAndconfig[] = await Promise.all(
      Object.entries(airspace.bondMarkets).map(async ([name, marketConfig]) => {
        const market = await BondMarket.load(program, marketConfig.bondManager, bondsProgramId);
        return { market, config: marketConfig, name };
      })
    );
    setMarkets(markets);
  };
  useEffect(() => {
    if (config?.bondsProgramId) {
      const program = new Program(JetBondsIdl, config.bondsProgramId, provider);
      const airspace = config.airspaces.find(airspace => airspace.name === 'default');
      loadBondMarkets(airspace, program, config.bondsProgramId);
    }
  }, [config]);
  return null;
};
