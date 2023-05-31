import { formatRate } from '@utils/format';
import { Skeleton, Typography } from 'antd';

interface ILendingRate {
  side: 'borrow' | 'deposit';
  pool: PoolData;
}
// Renders the borrow / deposit rates for the pool
export const LendingRate = ({ side, pool }: ILendingRate) => {
  // TODO eventually remove this atom
  const rate = side === 'borrow' ? pool.borrow_rate : pool.lending_rate;
  if (!isNaN(Number(rate))) {
    return (
      <Typography.Text type={side === 'borrow' ? 'danger' : 'success'}>{formatRate(Number(rate))}</Typography.Text>
    );
  }

  return <Skeleton className="align-right" paragraph={false} active />;
};
