import { useEffect, useMemo } from 'react';
import { atom, useRecoilValue, selector, selectorFamily, useSetRecoilState } from 'recoil';
import { PoolManager as MarginPoolManager, Pool } from '@jet-lab/margin';
import { localStorageEffect } from '../effects/localStorageEffect';
import { useProvider } from '@utils/jet/provider';
import { MainConfig } from '@state/config/marginConfig';
import { NetworkStateAtom } from '@state/network/network-state';
import { useJetStore } from '@jet-lab/store';
import { PoolData } from '@jet-lab/store/dist/slices/pools';
import { ActionRefresh } from '@state/actions/actions';

// Our app's interface for interacting with margin pools
export interface JetMarginPools {
  totalSupply: number;
  totalBorrowed: number;
  tokenPools: Record<string, Pool>;
}
// A simpler interface for when we're simply selecting a pool
export interface PoolOption {
  name: string | undefined;
  symbol: string | undefined;
}

// Pool Manager instantiation at app init
const PoolManager = atom({
  key: 'poolManager',
  default: undefined as MarginPoolManager | undefined,
  dangerouslyAllowMutability: true
});
// Overall state of all pools, fetched on init and on an ACTION_REFRESH_INTERVAL
export const Pools = atom({
  key: 'pools',
  default: undefined as JetMarginPools | undefined,
  dangerouslyAllowMutability: true
});
// Track the current pool by its symbol, so it's lightweight
// and we can reference this value to select the entire state
export const CurrentPoolSymbol = atom({
  key: 'currentPoolSymbol',
  default: 'BTC',
  effects: [localStorageEffect('jetAppCurrentPool')]
});

// Select the current pool's state
export const CurrentPool = selector<Pool | undefined>({
  key: 'currentPool',
  get: ({ get }) => {
    const pools = get(Pools);
    const symbol = get(CurrentPoolSymbol);

    const currentPool = pools?.tokenPools[symbol];
    return currentPool;
  },
  dangerouslyAllowMutability: true
});
// Return a simple list of pool options to choose from
export const PoolOptions = selector<PoolOption[]>({
  key: 'poolOptions',
  get: ({ get }) => {
    const config = get(MainConfig);
    if (!config) {
      return [];
    }

    return Object.values(config.tokens).map(token => ({
      name: token.name,
      symbol: token.symbol
    }));
  },
  dangerouslyAllowMutability: true
});
// Returns filtered pools from a query string
export const FilteredPools = selectorFamily<Pool[], string>({
  key: 'filteredPools',
  get:
    (filterText: string) =>
    ({ get }) => {
      const pools = get(Pools);
      if (!pools) {
        return [];
      }

      const filteredPools = Object.values(pools?.tokenPools).filter((pool: Pool) => !!pool.symbol);
      if (!filterText) {
        return filteredPools;
      } else {
        return filteredPools.filter(
          pool =>
            pool.symbol?.toLowerCase().includes(filterText.toLowerCase()) ||
            pool.name?.toLowerCase().includes(filterText.toLocaleLowerCase())
        );
      }
    },
  dangerouslyAllowMutability: true
});

// Get a pool from a given pool name
export function usePoolFromName(poolName: string | undefined): Pool | undefined {
  const pools = useRecoilValue(Pools);
  return useMemo(() => {
    if (pools && poolName) {
      return pools.tokenPools[poolName];
    }
    return undefined;
  }, [poolName, pools]);
}

// A syncer to be called so that we can have dependent atom state
export function usePoolsSyncer() {
  const { programs, provider } = useProvider();
  const setPoolManager = useSetRecoilState(PoolManager);
  const setPools = useSetRecoilState(Pools);
  const networkState = useRecoilValue(NetworkStateAtom);
  const initAllPools = useJetStore(state => state.initAllPools);
  const actionRefresh = useRecoilValue(ActionRefresh);

  // When we have an anchor provider, instantiate Pool Manager
  useEffect(() => {
    // Use pool manager to load pools state
    async function getPools(poolManager: MarginPoolManager) {
      const tokenPools = await poolManager.loadAll();
      let totalSupply = 0;
      let totalBorrowed = 0;
      const poolsToInit: Record<string, PoolData> = {};
      for (const pool of Object.values(tokenPools)) {
        const tokenPrice = tokenPools[pool.symbol].tokenPrice;
        const vault = tokenPools[pool.symbol].vault.tokens;
        const borrowedTokens = tokenPools[pool.symbol].borrowedTokens.tokens;

        totalSupply += vault * tokenPrice;
        totalBorrowed += borrowedTokens * tokenPrice;
        const address = pool.address.toBase58();

        poolsToInit[address] = {
          address: address,
          borrowed_tokens: borrowedTokens,
          deposit_tokens: vault,
          symbol: pool.symbol,
          token_mint: pool.tokenMint.toBase58(),
          decimals: pool.decimals,
          precision: pool.precision,
          collateral_weight: pool.depositNoteMetadata.valueModifier.toNumber(),
          collateral_factor: pool.loanNoteMetadata.valueModifier.toNumber()
        };
      }

      initAllPools(poolsToInit);

      setPools({
        totalSupply,
        totalBorrowed,
        tokenPools
      });
    }

    if (programs && provider && networkState === 'connected') {
      const poolManager = new MarginPoolManager(programs, provider);
      setPoolManager(poolManager);

      // Use manager to fetch pools on an interval
      getPools(poolManager);
    }

    // TODO remove resetting pools upon action
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [programs, provider.connection, actionRefresh, networkState]);
}
