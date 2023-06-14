import { Number192 } from '../utils/Number192';
import { StateCreator } from 'zustand';

const interpolate = (x: number, x0: number, x1: number, y0: number, y1: number): number => {
  return y0 + ((x - x0) * (y1 - y0)) / (x1 - x0);
};

const getCcRate = (reserveConfig: PoolRateConfig, utilRate: number): number => {
  const basisPointFactor = 10000;
  const util1 = reserveConfig.utilizationRate1 / basisPointFactor;
  const util2 = reserveConfig.utilizationRate2 / basisPointFactor;
  const borrow0 = reserveConfig.borrowRate0 / basisPointFactor;
  const borrow1 = reserveConfig.borrowRate1 / basisPointFactor;
  const borrow2 = reserveConfig.borrowRate2 / basisPointFactor;
  const borrow3 = reserveConfig.borrowRate3 / basisPointFactor;

  if (utilRate <= util1) {
    return interpolate(utilRate, 0, util1, borrow0, borrow1);
  } else if (utilRate <= util2) {
    return interpolate(utilRate, util1, util2, borrow1, borrow2);
  } else {
    return interpolate(utilRate, util2, 1, borrow2, borrow3);
  }
};

export const createPoolsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], PoolsSlice> = set => ({
  pools: {},
  selectedPoolKey: '',
  poolsLastUpdated: 0,
  updatePool: (update: PoolDataUpdate) => {
    return set(
      state => {
        const pool = state.pools[update.address];
        const borrowed_tokens = Number192.fromBits(update.borrowed_tokens).toNumber() / 10 ** pool.decimals;
        const deposit_tokens = update.deposit_tokens / 10 ** pool.decimals;
        const util_ratio = borrowed_tokens / deposit_tokens;
        const ccRate = getCcRate(pool.pool_rate_config, util_ratio);
        return {
          pools: {
            ...state.pools,
            [update.address]: {
              ...pool,
              borrowed_tokens,
              deposit_tokens,
              borrow_rate: ccRate,
              lending_rate: (1 - pool.pool_rate_config.managementFeeRate) * ccRate * util_ratio
            }
          },
          poolsLastUpdated: Date.now()
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
  },
  selectPool: (address: string) => set(() => ({ selectedPoolKey: address }), false, 'SELECT_POOL')
});
