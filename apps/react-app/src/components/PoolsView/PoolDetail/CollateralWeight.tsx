import { formatRate } from '@utils/format';
import { Skeleton, Typography } from 'antd';
import { WithPoolData } from './PoolDetail';

// Renders the collateral weight for the current pool
export const CollateralWeight = ({ selectedPool }: WithPoolData) => {
  if (selectedPool) {
    return <Typography.Text>{formatRate(selectedPool.collateral_weight)}</Typography.Text>;
  }
  return <Skeleton paragraph={false} active style={{ width: 100 }} />;
};
