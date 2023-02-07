import { formatRate } from '@utils/format';
import { WithPoolData } from './PoolDetail';
import { Typography } from 'antd';

// Renders the utilization rate of the current pool
export const UtilizationRate = ({ selectedPool }: WithPoolData) => {
  let rateString = '—%';
  if (selectedPool && selectedPool.deposit_tokens > 0) {
    rateString = formatRate(selectedPool.borrowed_tokens / selectedPool.deposit_tokens);
  }
  return <Typography.Text type="secondary" italic>{`Utilization Rate ${rateString}`}</Typography.Text>;
};
