import { FixedTermMarket, JetFixedTermIdl, MarketAndConfig, OrderbookModel } from '@jet-lab/margin';
import { Program } from '@project-serum/anchor';
import { useEffect } from 'react';
import { atom, selector, useRecoilValue, useSetRecoilState } from 'recoil';
import { AirspaceConfig } from '@jet-lab/margin';
import { MainConfig } from '../config/marginConfig';
import { PublicKey } from '@solana/web3.js';
import { useProvider } from '@utils/jet/provider';
import { NetworkStateAtom } from '@state/network/network-state';
import { useLocation } from 'react-router-dom';
import { getOrderbookSnapshot, useJetStore } from '@jet-lab/store';

export const AllFixedTermMarketsAtom = atom<Array<MarketAndConfig>>({
  key: 'allFixedTermMarkets',
  default: [],
  dangerouslyAllowMutability: true
});

export const SelectedFixedTermMarketAtom = atom<number>({
  key: 'selectedFixedTermMarketIndex',
  default: 0,
  dangerouslyAllowMutability: true
});

export const FixedTermMarketAtom = selector<MarketAndConfig | null>({
  key: 'fixedTermMarketAtom',
  get: ({ get }) => {
    const list = get(AllFixedTermMarketsAtom);
    const selected = get(SelectedFixedTermMarketAtom);
    return list[selected];
  },
  dangerouslyAllowMutability: true
});

export type CurrentOrderTab = 'borrow-now' | 'lend-now' | 'offer-loan' | 'request-loan' | 'not_set';

export const CurrentOrderTabAtom = atom<CurrentOrderTab>({
  key: 'current-fixed-term-order-tab',
  default: 'not_set'
});

export interface ExtendedOrderBook {
  name: string;
  orderbook: OrderbookModel;
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
    program: Program<JetFixedTermIDL>,
    marginProgramId: PublicKey
  ) => {
    const markets: Array<MarketAndConfig> = [];
    await Promise.all(
      Object.entries(airspace.fixedTermMarkets).map(async ([name, marketConfig]) => {
        try {
          const market = await FixedTermMarket.load(program, marketConfig.market, marginProgramId);
          const token = Object.values(config?.tokens || {}).find(token => token.symbol === marketConfig.symbol);
          if (token) {
            markets.push({ market, config: marketConfig, name, token });
          }
        } catch (e) {
          console.log(e);
        }
      })
    );
    setMarkets(markets.sort((a, b) => b.name.localeCompare(a.name)));
  };

  useEffect(() => {
    if (networkState === 'connected' && config?.fixedTermMarketProgramId) {
      const program = new Program(JetFixedTermIdl, config.fixedTermMarketProgramId, provider);
      const airspace = config.airspaces.find(airspace => airspace.name === 'default') || config.airspaces[0];
      if (airspace) {
        loadFixedTermMarkets(airspace, program, new PublicKey(config.marginProgramId));
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
