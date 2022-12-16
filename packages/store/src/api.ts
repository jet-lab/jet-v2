import { MarginAccount } from '@jet-lab/margin';
import { FixedTermMarket } from '@jet-lab/margin';
import { FixedOrder } from './types';
import useSWR from 'swr';

export const useOrdersForUser = (market?: FixedTermMarket, account?: MarginAccount) => {
  const path = `fixed/orders/${market?.address}/${account?.address}`;
  const {
    data,
    error,
    mutate: refresh
  } = useSWR<FixedOrder[]>(path, async () => {
    if (account && market) {
      return fetch(`${process.env.DATA_API}/${path}`).then(r => r.json());
    } else {
      return [];
    }
  });

  return {
    data,
    error,
    refresh,
    loading: !data && !error
  };
};
