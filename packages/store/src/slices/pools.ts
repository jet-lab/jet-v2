import { Pool } from '@jet-lab/margin';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';

export interface PoolsSlice {
  pools: Record<string, Pool>;
  updatePool: (update: PoolUpdate) => void;
}

export interface PoolUpdate {
  address: string;
  borrowed_tokens: number;
  deposit_tokens: number;
  deposit_notes: number;
  accrued_until: Date;
}

export const createPoolsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], PoolsSlice> = set => ({
  pools: {},
  updatePool: (update: PoolUpdate) => {
    console.log(update);
    return set(
      state => {
        return {
          ...state,
          pools: {
            ...state.pools
          }
        };
      },
      false,
      'UPDATE_POOL'
    );
  }
});
