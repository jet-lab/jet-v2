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
  const data: OrderbookSnapshot = await fetch(`${apiEndpoint}/${path}`).then(r => r.json());
  if (data.bids.length === 0 && data.asks.length === 0) {
    console.warn('No liquidity either side of the market. Is this a new market?');
  }
  return data;
};
