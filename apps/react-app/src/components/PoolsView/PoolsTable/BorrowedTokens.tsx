import { useCurrencyFormatting } from '@utils/currency';
import { Skeleton, Typography } from 'antd';
import { WithPoolData } from '../PoolDetail/PoolDetail';

// Renders the borrowed tokens for the pool
export const BorrowedTokens = ({ pool }: WithPoolData) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  if (pool) {
    const tokensAbbrev = currencyAbbrev(pool.borrowed_tokens, pool.precision, false);
    return <Typography.Text>{`${tokensAbbrev}`}</Typography.Text>;
  }

  return <Skeleton className="align-right" paragraph={false} active />;
};
