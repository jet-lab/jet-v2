import { useEffect, useMemo } from 'react';
import { atom, useRecoilState, useSetRecoilState, useRecoilValue, selector } from 'recoil';
import { PoolManager, Pool } from '@jet-lab/margin';
import { localStorageEffect } from '../effects/localStorageEffect';
import { Dictionary } from '../settings/localization/localization';
import { ActionRefresh, ACTION_REFRESH_INTERVAL } from '../actions/actions';
import { useProvider } from '../../utils/jet/provider';
import { notify } from '../../utils/notify';
import { useMarginConfig } from '../../utils/jet/marginConfig';

export interface JetMarginPools {
  totalSupply: number;
  totalBorrowed: number;
  tokenPools: Record<string, Pool>;
}

export interface PoolWithSymbol extends Pool {
  symbol: string;
}

export const Pools = atom({
  key: 'pools',
  default: undefined as JetMarginPools | undefined,
  dangerouslyAllowMutability: true
});
export const CurrentPool = atom({
  key: 'currentPool',
  default: undefined as Pool | undefined,
  dangerouslyAllowMutability: true
});
export const CurrentPoolSymbol = atom({
  key: 'currentPoolSymbol',
  default: 'BTC',
  effects: [localStorageEffect('jetAppCurrentPool')]
});
export const PoolOptions = atom({
  key: 'poolOptions',
  default: [] as { name: string; symbol: string }[]
});
export const PoolsInit = atom({
  key: 'poolsInit',
  default: false as boolean
});

export const PoolsTextFilter = atom({
  key: 'poolsTextFilter',
  default: ''
});

export const FilteredPoolsList = selector<PoolWithSymbol[]>({
  key: 'filteredPoolsList',
  get: ({ get }) => {
    const filter = get(PoolsTextFilter);
    const list = get(Pools);

    if (!list) {
      return [];
    }

    const validPools = Object.values(list?.tokenPools).filter((pool: Pool): pool is PoolWithSymbol => !!pool.symbol);

    if (!filter) {
      return validPools;
    } else {
      return validPools.filter(
        pool =>
          pool.symbol?.toLowerCase().includes(filter.toLowerCase()) ||
          pool.name?.toLowerCase().includes(filter.toLocaleLowerCase())
      );
    }
  }
});

// Wrapper to provide contextual updates to Pools
export function PoolsWrapper(props: { children: JSX.Element }) {
  const { programs, provider } = useProvider();
  const dictionary = useRecoilValue(Dictionary);
  const config = useMarginConfig();
  const [pools, setPools] = useRecoilState(Pools);
  const setCurrentPool = useSetRecoilState(CurrentPool);
  const currentPoolSymbol = useRecoilValue(CurrentPoolSymbol);
  const setPoolOptions = useSetRecoilState(PoolOptions);
  const [poolsInit, setPoolsInit] = useRecoilState(PoolsInit);
  const actionRefresh = useRecoilValue(ActionRefresh);

  // Set poolOptions and marginTokenOptions on init
  useEffect(() => {
    if (!config) {
      return;
    }

    const poolOptions: { name: string; symbol: string }[] = [];
    for (const token of Object.values(config.tokens)) {
      poolOptions.push({
        name: token.name,
        symbol: token.symbol
      });
    }
    setPoolOptions(poolOptions);
  }, [config, setPoolOptions]);

  // On init and currentPoolSymbol change, update currentPool
  useEffect(() => {
    if (pools) {
      setCurrentPool(pools.tokenPools[currentPoolSymbol]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [poolsInit, currentPoolSymbol]);

  // Fetch Jet pools on init
  // Re-fetch upon an actionRefresh
  useEffect(() => {
    async function getPools() {
      if (!programs) {
        return;
      }
      try {
        const poolManager = new PoolManager(programs, provider);
        const tokenPools = await poolManager.loadAll();
        let totalSupply = 0;
        let totalBorrowed = 0;
        for (const token of Object.values(tokenPools)) {
          if (!token.symbol) {
            return;
          }

          const tokenPrice = tokenPools[token.symbol].tokenPrice;
          const vault = tokenPools[token.symbol].vault.tokens;
          const borrowedTokens = tokenPools[token.symbol].borrowedTokens.tokens;

          totalSupply += vault * tokenPrice;
          totalBorrowed += borrowedTokens * tokenPrice;
        }

        setPools({
          totalSupply,
          totalBorrowed,
          tokenPools
        });
        setPoolsInit(true);
      } catch (err) {
        console.error(err);
        notify(dictionary.poolsView.errors.noPools, dictionary.poolsView.errors.noPoolsDetail, 'error');
      }
    }

    getPools();
    const poolsInterval = setInterval(getPools, ACTION_REFRESH_INTERVAL);
    return () => clearInterval(poolsInterval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [provider.connection, programs, actionRefresh]);

  return <>{props.children}</>;
}

// Get a pool from a given pool name
export function usePoolFromName(poolName: string | undefined): Pool | undefined {
  const pools = useRecoilValue(Pools);
  const poolsInit = useRecoilValue(PoolsInit);
  return useMemo(() => {
    if (poolsInit && pools && poolName) {
      return pools.tokenPools[poolName];
    }
  }, [poolName, pools, poolsInit]);
}
