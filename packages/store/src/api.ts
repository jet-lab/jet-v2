import { OpenOrders, OpenPositions, OrderbookSnapshot, SwapLiquidity } from './types';
import useSWR, { SWRResponse } from 'swr';

export const useOrdersForUser = (apiEndpoint: string, market?: string, account?: string): SWRResponse<OpenOrders> => {
  const path = `fixed/open-orders/${market}/${account}`;
  return useSWR<OpenOrders>(
    path,
    async () => {
      if (account && market) {
        return fetch(`${apiEndpoint}/${path}`).then(r => r.json());
      } else {
        return [];
      }
    },
    { refreshInterval: 30_000 }
  );
};

export const useOpenPositions = (
  apiEndpoint: string,
  market?: string,
  account?: string
): SWRResponse<OpenPositions> => {
  const path = `fixed/open-positions/${market}/${account}`;
  return useSWR<OpenPositions>(
    path,
    async () => {
      if (account && market) {
        return fetch(`${apiEndpoint}/${path}`).then(r => r.json());
      } else {
        return [];
      }
    },
    { refreshInterval: 30_000 }
  );
};

export const getOrderbookSnapshot = async (apiEndpoint: string, market: string): Promise<OrderbookSnapshot> => {
  const path = `fixed/orderbook-snapshot/${market}`;
  const data = await fetch(`${apiEndpoint}/${path}`).then(r => r.json());
  return data;
};

export const getSwapLiquidity = (
  apiEndpoint: string,
  from: string,
  to: string,
  amount: number
): SWRResponse<SwapLiquidity | null> => {
  const path = `${apiEndpoint}/swap/liquidity/${from}/${to}/${amount}`;
  return useSWR<SwapLiquidity | null>(path, async () => fetch(path).then(r => r.json()), { refreshInterval: 30_000 });
};
