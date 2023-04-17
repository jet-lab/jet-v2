import { useEffect, useMemo } from 'react';
import { atom, useRecoilValue, useSetRecoilState } from 'recoil';
import { TokenAmount } from '@jet-lab/margin';
import { ActionRefresh, ACTION_REFRESH_INTERVAL, CurrentSwapOutput, TokenInputAmount } from '../actions/actions';
import { useProvider } from '@utils/jet/provider';
import { Pools } from '../pools/pools';

import { useJetStore } from '@jet-lab/store';

import { getSwapRoutes, SwapQuote } from '@utils/actions/swap';


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
export const SwapQuotes = atom({
  key: 'swapQuotes',
  default: [] as SwapQuote[]
});
export const SelectedSwapQuote = atom({
  key: 'selectedSwapQuote',
  default: undefined as SwapQuote | undefined
});
export function useSplSwapSyncer() {
  const cluster = useJetStore(state => state.settings.cluster);
  const { provider } = useProvider();
  const selectedPoolKey = useJetStore(state => state.selectedPoolKey);
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );
  const outputToken = useRecoilValue(CurrentSwapOutput);
  const setSwapPoolTokenAmounts = useSetRecoilState(SwapPoolTokenAmounts);
  const setSwapRoutes = useSetRecoilState(SwapQuotes);
  const setSwapFees = useSetRecoilState(SwapFees);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const swapEndpoint = cluster === "mainnet-beta" ? "" : cluster === "devnet" ? process.env.REACT_APP_DEV_SWAP_API : process.env.REACT_APP_LOCAL_SWAP_API;

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
        const routes = await getSwapRoutes(swapEndpoint || "", currentPool.tokenMint, outputToken.tokenMint, tokenInputAmount);
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
