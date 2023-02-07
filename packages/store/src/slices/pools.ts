import BN from 'bn.js';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

export interface PoolsSlice {
  pools: Record<string, PoolData>;
  selectedPoolKey: string;
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
  selected?: boolean;
  precision: number;
  collateral_weight: number;
  collateral_factor: number;
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
  selectedPoolKey: '',
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
    // on init select first pool if no other pool is selected
    const keys = Object.keys(update);
    return set(
      state => ({
        ...state,
        pools: update,
        selectedPoolKey: keys.includes(String(state.selectedPoolKey)) ? state.selectedPoolKey : keys[0]
      }),
      true,
      'INIT_POOLS'
    );
  }
});
