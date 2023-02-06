import BN from 'bn.js';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

export interface PoolsSlice {
  pools: Record<string, PoolData>;
  updatePool: (update: PoolDataUpdate) => void;
  initAllPools: (update: Record<string, PoolData>) => void;
}

export interface PoolData {
  address: string;
  borrowed_tokens: number;
  deposit_tokens: number;
  symbol: string;
  token_mint: string;
  decimals: number;
  // deposit_notes: number;
  // accrued_until: Date;
}

export interface PoolDataUpdate {
  address: string;
  borrowed_tokens: number[];
  deposit_tokens: number;
  // deposit_notes: number;
  // accrued_until: Date;
}

export const createPoolsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], PoolsSlice> = set => ({
  pools: {},
  updatePool: (update: PoolDataUpdate) => {
    return set(
      state => {
        const pool = state.pools[update.address];
        return {
          pools: {
            ...state.pools,
            [update.address]: {
              ...pool,
              borrowed_tokens: new BN(update.borrowed_tokens).toNumber() / 10 ** pool.decimals,
              deposit_tokens: new BN(update.deposit_tokens).toNumber() / 10 ** pool.decimals
            }
          }
        };
      },
      false,
      'UPDATE_POOL'
    );
  },
  initAllPools: (update: Record<string, PoolData>) => {
    return set(
      () => ({
        pools: update
      }),
      false,
      'INIT_POOLS'
    );
  }
});
