interface PoolsSlice {
  pools: Record<string, PoolData>;
  selectedPoolKey: string;
  poolsLastUpdated: number;
  updatePool: (update: PoolDataUpdate) => void;
  initAllPools: (update: Record<string, PoolData>) => void;
  selectPool: (address: string) => void;
}

interface PoolData {
  address: string;
  name: string;
  borrowed_tokens: number;
  deposit_tokens: number;
  symbol: string;
  token_mint: string;
  decimals: number;
  selected?: boolean;
  precision: number;
  collateral_weight: number;
  collateral_factor: number;
  pool_rate_config: PoolRateConfig;
  lending_rate: number;
  borrow_rate: number;
  // deposit_notes: number;
  // accrued_until: Date;
}

interface PoolDataUpdate {
  address: string;
  borrowed_tokens: number[];
  deposit_tokens: number;
  // deposit_notes: number;
  // accrued_until: Date;
}

interface PoolRateConfig {
  utilizationRate1: number;
  utilizationRate2: number;
  borrowRate0: number;
  borrowRate1: number;
  borrowRate2: number;
  borrowRate3: number;
  managementFeeRate: number;
}
