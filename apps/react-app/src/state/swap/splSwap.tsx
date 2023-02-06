import { useEffect } from 'react';
import { atom, useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { SPLSwapPool, TokenAmount } from '@jet-lab/margin';
import { ActionRefresh, ACTION_REFRESH_INTERVAL, CurrentSwapOutput } from '../actions/actions';
import { useProvider } from '@utils/jet/provider';
import { CurrentPool } from '../pools/pools';
import { getSwapPoolPrice } from '@utils/actions/swap';

import orcaPools from '@jet-lab/margin/src/margin/swap/orca-swap-pools.json';
import { MainConfig } from '@state/config/marginConfig';
import { useJetStore } from '@jet-lab/store';

// Market
export const SplSwapPools = atom({
  key: 'splSwapPools',
  default: {} as Record<string, SPLSwapPool>
});
export const CurrentSplSwapPool = atom({
  key: 'currentSplSwapPool',
  default: undefined as
    | {
        pool: SPLSwapPool;
        inverted: boolean;
      }
    | undefined
});
export const SwapPair = atom({
  key: 'swapPair',
  default: undefined as string | undefined
});
export const SwapFees = atom({
  key: 'swapFees',
  default: 0.0 as number
});
export const SwapPoolTokenAmounts = atom({
  key: 'swapPoolTokenAmounts',
  default: undefined as
    | {
        source: TokenAmount;
        destination: TokenAmount;
      }
    | undefined
});
export function useSplSwapSyncer() {
  const cluster = useJetStore(state => state.settings.cluster);
  const { provider } = useProvider();
  const config = useRecoilValue(MainConfig);
  const currentPool = useRecoilValue(CurrentPool);
  const outputToken = useRecoilValue(CurrentSwapOutput);
  const [swapPools, setSwapPools] = useRecoilState(SplSwapPools);
  const [currentSwapPool, setCurrentSwapPool] = useRecoilState(CurrentSplSwapPool);
  const setSwapPoolTokenAmounts = useSetRecoilState(SwapPoolTokenAmounts);
  const setSwapFees = useSetRecoilState(SwapFees);
  const actionRefresh = useRecoilValue(ActionRefresh);

  // Setup swap pools on init
  useEffect(() => {
    if (cluster === 'devnet') {
      if (config && config.exchanges) {
        setSwapPools(config.exchanges);
      }
    } else {
      // @ts-ignore
      setSwapPools(orcaPools);
    }
  }, [config]);

  // Set the swap pool when input or output tokens change
  useEffect(() => {
    // Reset the pool token amounts to force charts to wait for fresh data
    if (!currentPool || !outputToken) {
      // Clear the pool
      setSwapPoolTokenAmounts(undefined);
      setCurrentSwapPool(undefined);
      setSwapFees(0.0);
      return;
    }

    setSwapPoolTokenAmounts(undefined);
    // Check if the direct swap pool exists
    const key = `${currentPool.symbol}/${outputToken.symbol}`;
    const inverseKey = `${outputToken.symbol}/${currentPool.symbol}`;
    if (swapPools[key]) {
      const pool = swapPools[key];
      setCurrentSwapPool({
        pool,
        inverted: false
      });
      setSwapFees(pool.swapFees);
    } else if (swapPools[inverseKey]) {
      const pool = swapPools[inverseKey];
      setCurrentSwapPool({
        pool,
        inverted: true
      });
      setSwapFees(pool.swapFees);
    } else {
      setCurrentSwapPool(undefined);
      setSwapFees(0.0);
    }

    // Get pool prices and set a timer to refresh them
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentPool?.symbol, outputToken?.symbol, swapPools]);

  // Fetch prices on pool pair change
  // Re-fetch on actionRefresh
  useEffect(() => {
    async function getSwapTokenPrices() {
      if (!currentSwapPool) {
        return;
      }

      try {
        const prices = await getSwapPoolPrice(provider, currentSwapPool.pool);
        if (!currentSwapPool.inverted) {
          setSwapPoolTokenAmounts({
            source: prices.balanceTokenA,
            destination: prices.balanceTokenB
          });
        } else {
          setSwapPoolTokenAmounts({
            source: prices.balanceTokenB,
            destination: prices.balanceTokenA
          });
        }
      } catch (err) {
        console.error(err);
      }
    }

    getSwapTokenPrices();
    const swapPricesInterval = setInterval(getSwapTokenPrices, ACTION_REFRESH_INTERVAL);
    return () => clearInterval(swapPricesInterval);
  }, [provider, cluster, currentSwapPool, actionRefresh, setSwapPoolTokenAmounts]);

  return <></>;
}

// Check if a given pair has a corresponding Orca pool
export function hasOrcaPool(swapPools: Record<string, SPLSwapPool>, inputSymbol: string, outputSymbol: string) {
  const pair = `${inputSymbol}/${outputSymbol}`;
  const inversePair = `${outputSymbol}/${inputSymbol}`;

  return Object.keys(swapPools).includes(pair) || Object.keys(swapPools).includes(inversePair);
}
