import { MarginAccount, OrderbookSnapshot } from '@jet-lab/margin';
import { FixedTermMarket } from '@jet-lab/margin';
import { OpenOrders, OpenPositions } from './types';
import useSWR, { SWRResponse } from 'swr';

export const useOrdersForUser = (apiEndpoint: string, market?: FixedTermMarket, account?: MarginAccount): SWRResponse<OpenOrders> => {
  const path = `fixed/open-orders/${market?.address}/${account?.address}`;
  return useSWR<OpenOrders>(path, async () => {
    if (account && market) {
      return fetch(`${apiEndpoint}/${path}`).then(r => {
        const out = r.json();
        console.log('open-orders', out);
        return out
      });
    } else {
      return null;
    }
  }, { refreshInterval: 30_000 });
};

export const useOpenPositions = (apiEndpoint: string, market?: FixedTermMarket, account?: MarginAccount): SWRResponse<OpenPositions> => {
  const path = `fixed/open-positions/${market?.address}/${account?.address}`;
  return useSWR<OpenPositions>(path, async () => {
    if (account && market) {
      return fetch(`${apiEndpoint}/${path}`).then(r => r.json());
    } else {
      return null;
    }
  }, { refreshInterval: 30_000 });
};

export const getOrderbookSnapshot = async (
  apiEndpoint: string,
  market: FixedTermMarket
): Promise<OrderbookSnapshot> => {
  const path = `fixed/orderbook-snapshot/${market.address}`;
  const data = await fetch(`${apiEndpoint}/${path}`).then(r => r.json())
  return data
};
