import { useCurrencyFormatting } from '@utils/currency';
import { Skeleton, Typography } from 'antd';
import { WithPoolData } from '../PoolDetail/PoolDetail';

// Renders the available liquidity for the pool
export const AvailableLiquidity = ({ pool }: WithPoolData) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  if (pool) {
    const tokensAbbrev = currencyAbbrev(pool.deposit_tokens, pool.precision, false);
    return <Typography.Text>{`${tokensAbbrev}`}</Typography.Text>;
  }

  return <Skeleton className="align-right" paragraph={false} active />;
};
