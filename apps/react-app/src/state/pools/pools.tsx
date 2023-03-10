import { useEffect } from 'react';
import { atom, useRecoilValue, selector, useSetRecoilState } from 'recoil';
import { PoolManager as MarginPoolManager, Pool } from '@jet-lab/margin';
import { useProvider } from '@utils/jet/provider';
import { MainConfig } from '@state/config/marginConfig';
import { NetworkStateAtom } from '@state/network/network-state';
import { useJetStore } from '@jet-lab/store';
import { PoolData } from '@jet-lab/store/dist/slices/pools';

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

// Overall state of all pools, fetched on init and on an ACTION_REFRESH_INTERVAL
export const Pools = atom({
  key: 'pools',
  default: undefined as JetMarginPools | undefined,
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

// A syncer to be called so that we can have dependent atom state
export function usePoolsSyncer() {
  const { programs, provider } = useProvider();
  const setPools = useSetRecoilState(Pools);
  const networkState = useRecoilValue(NetworkStateAtom);
  const { initAllPools, poolsLastUpdated } = useJetStore(state => ({ initAllPools: state.initAllPools, poolsLastUpdated: state.poolsLastUpdated }));

  // When we have an anchor provider, instantiate Pool Manager
  useEffect(() => {
    // Use pool manager to load pools state

    let isLoading = false;
    async function getPools() {
      if (!programs) return;

      const poolManager = new MarginPoolManager(programs, provider);
      const tokenPools = await poolManager.loadAll();

      if (isLoading) {
        return;
      }
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

        const pool_rate_config = {
          utilizationRate1: Number(pool.info?.marginPool.config.utilizationRate1),
          utilizationRate2: Number(pool.info?.marginPool.config.utilizationRate2),
          borrowRate0: Number(pool.info?.marginPool.config.borrowRate0),
          borrowRate1: Number(pool.info?.marginPool.config.borrowRate1),
          borrowRate2: Number(pool.info?.marginPool.config.borrowRate2),
          borrowRate3: Number(pool.info?.marginPool.config.borrowRate3),
          managementFeeRate: Number(pool.info?.marginPool.config.managementFeeRate)
        };

        poolsToInit[address] = {
          address: address,
          name: pool.name,
          borrowed_tokens: borrowedTokens,
          deposit_tokens: vault,
          symbol: pool.symbol,
          token_mint: pool.tokenMint.toBase58(),
          decimals: pool.decimals,
          precision: pool.precision,
          collateral_weight: pool.depositNoteMetadata.valueModifier.toNumber(),
          collateral_factor: pool.loanNoteMetadata.valueModifier.toNumber(),
          pool_rate_config,
          lending_rate: pool.depositApy,
          borrow_rate: pool.borrowApr
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
      getPools();
    }

    return () => {
      isLoading = true;
    };
    // TODO remove resetting pools upon action
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [programs?.config, networkState, poolsLastUpdated]);
}
