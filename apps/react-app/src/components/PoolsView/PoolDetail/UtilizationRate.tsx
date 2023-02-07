import { formatRate } from '@utils/format';
import { WithPoolData } from './PoolDetail';
import { Typography } from 'antd';

// Renders the utilization rate of the current pool
export const UtilizationRate = ({ pool }: WithPoolData) => {
  let rateString = '—%';
  if (pool && pool.deposit_tokens > 0) {
    rateString = formatRate(pool.borrowed_tokens / pool.deposit_tokens);
  }
  return <Typography.Text type="secondary" italic>{`Utilization Rate ${rateString}`}</Typography.Text>;
};
