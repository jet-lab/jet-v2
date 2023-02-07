import { useCurrencyFormatting } from '@utils/currency';
import { WithPoolData } from './PoolDetail';
import { Skeleton, Typography } from 'antd';

export // Renders the pool size for the current pool
const PoolSize = ({ pool }: WithPoolData) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  if (pool) {
    const totalValueAbbrev = currencyAbbrev(pool.deposit_tokens, pool.precision, false, undefined);
    return <Typography.Title className="green-text">{`${totalValueAbbrev}`}</Typography.Title>;
  }

  return <Skeleton className="align-center" paragraph={false} active style={{ margin: '10px 0' }} />;
};
