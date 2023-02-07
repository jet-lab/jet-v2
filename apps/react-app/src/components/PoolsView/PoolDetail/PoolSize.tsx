import { useCurrencyFormatting } from '@utils/currency';
import { WithPoolData } from './PoolDetail';
import { Skeleton, Typography } from 'antd';

export // Renders the pool size for the current pool
const PoolSize = ({ selectedPool }: WithPoolData) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  if (selectedPool) {
    const totalValueAbbrev = currencyAbbrev(selectedPool.deposit_tokens, selectedPool.precision, false, undefined);
    return <Typography.Title className="green-text">{`${totalValueAbbrev}`}</Typography.Title>;
  }

  return <Skeleton className="align-center" paragraph={false} active style={{ margin: '10px 0' }} />;
};
