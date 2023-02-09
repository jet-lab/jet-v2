import { formatRate } from '@utils/format';
import { Skeleton, Typography } from 'antd';
import { WithPoolData } from '../PoolDetail/PoolDetail';

// Renders the utilization rate for the pool
export const UtilizationRate = ({ pool }: WithPoolData) => {
  if (pool) {
    return (
      <Typography.Text>
        {pool.deposit_tokens ? formatRate(pool.borrowed_tokens / (pool.deposit_tokens + pool.borrowed_tokens)) : '-'}
      </Typography.Text>
    );
  }

  return <Skeleton className="align-right" paragraph={false} active />;
};
