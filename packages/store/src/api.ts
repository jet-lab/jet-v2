import { MarginAccount } from '@jet-lab/margin';
import { FixedTermMarket } from '@jet-lab/margin';
import { OpenOrders, OpenPositions } from './types';
import useSWR from 'swr';

export const useOrdersForUser = (apiEndpoint: string, market?: FixedTermMarket, account?: MarginAccount) => {
  const path = `fixed/open-orders/${market?.address}/${account?.address}`;
  return useSWR<OpenOrders>(path, async () => {
    if (account && market) {
      return fetch(`${apiEndpoint}/${path}`).then(r => r.json());
    } else {
      return null;
    }
  });
};

export const useOpenPositions = (apiEndpoint: string, market?: FixedTermMarket, account?: MarginAccount) => {
  const path = `fixed/open-positions/${market?.address}/${account?.address}`;
  return useSWR<OpenPositions>(path, async () => {
    if (account && market) {
      return fetch(`${apiEndpoint}/${path}`).then(r => r.json());
    } else {
      return null;
    }
  });
};
