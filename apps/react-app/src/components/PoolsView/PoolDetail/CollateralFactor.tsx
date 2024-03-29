import { Skeleton, Typography } from 'antd';
import { WithPoolData } from './PoolDetail';

// Renders the required collateral factor for the current pool
export const CollateralFactor = ({ pool }: WithPoolData) =>
  pool ? (
    <Typography.Text>{pool.collateral_factor}</Typography.Text>
  ) : (
    <Skeleton paragraph={false} active style={{ width: 100 }} />
  );
