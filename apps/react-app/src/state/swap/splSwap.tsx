import { useEffect, useMemo } from 'react';
import { atom, useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { SPLSwapPool, TokenAmount } from '@jet-lab/margin';
import { ActionRefresh, ACTION_REFRESH_INTERVAL, CurrentSwapOutput, TokenInputAmount } from '../actions/actions';
import { useProvider } from '@utils/jet/provider';
import { Pools } from '../pools/pools';

import { MainConfig } from '@state/config/marginConfig';
import { useJetStore } from '@jet-lab/store';

import { getSwapRoutes, SwapRoute } from '@utils/actions/swap';


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
export const SwapRoutes = atom({
  key: 'swapRoutes',
  default: [] as SwapRoute[]
});
export function useSplSwapSyncer() {
  const cluster = useJetStore(state => state.settings.cluster);
  const { provider } = useProvider();
  const config = useRecoilValue(MainConfig);
  const selectedPoolKey = useJetStore(state => state.selectedPoolKey);
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );
  const outputToken = useRecoilValue(CurrentSwapOutput);
  const setSwapPoolTokenAmounts = useSetRecoilState(SwapPoolTokenAmounts);
  const setSwapRoutes = useSetRecoilState(SwapRoutes);
  const setSwapFees = useSetRecoilState(SwapFees);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);

  // Set the swap pool when input or output tokens change
  useEffect(() => {
    // Reset the pool token amounts to force charts to wait for fresh data
    if (!currentPool || !outputToken) {
      // Clear the pool
      setSwapPoolTokenAmounts(undefined);
      setSwapFees(0.0);
      return;
    }

    setSwapPoolTokenAmounts(undefined);

    // Get pool prices and set a timer to refresh them
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentPool?.symbol, outputToken?.symbol]);

  // Fetch prices on pool pair change
  // Re-fetch on actionRefresh
  useEffect(() => {
    if (!currentPool || !outputToken) {
      return;
    }
    async function getSwapTokenPrices() {
      if (!currentPool || !outputToken) {
        return;
      }
      try {
        const routes = await getSwapRoutes(currentPool.tokenMint, outputToken.tokenMint, tokenInputAmount);
        if (!routes) {
          return;
        }
        setSwapRoutes(routes);
      } catch (err) {
        console.error(err);
      }
    }

    getSwapTokenPrices();
    const swapPricesInterval = setInterval(getSwapTokenPrices, ACTION_REFRESH_INTERVAL);
    return () => clearInterval(swapPricesInterval);
  }, [
    provider,
    cluster,
    actionRefresh,
    tokenInputAmount,
    setSwapPoolTokenAmounts,
    currentPool?.symbol,
    outputToken?.symbol
  ]);

  return <></>;
}
