import { formatRate } from '@utils/format';
import { Skeleton, Typography } from 'antd';
import { WithPoolData } from './PoolDetail';

// Renders the collateral weight for the current pool
export const CollateralWeight = ({ pool }: WithPoolData) => {
  if (pool) {
    return <Typography.Text>{formatRate(pool.collateral_weight)}</Typography.Text>;
  }
  return <Skeleton paragraph={false} active style={{ width: 100 }} />;
};
