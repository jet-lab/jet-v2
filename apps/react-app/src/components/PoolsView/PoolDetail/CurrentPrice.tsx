import { useCurrencyFormatting } from '@utils/currency';
import { Skeleton, Typography } from 'antd';
import { WithPoolData } from './PoolDetail';

// Renders the current price of the current pool
export const CurrentPrice = ({ selectedPool, price }: WithPoolData & { price?: number }) => {
  const { currencyFormatter } = useCurrencyFormatting();
  if (selectedPool && price) {
    return <Typography.Text>{`1 ${selectedPool.symbol} = ${currencyFormatter(price, true)}`}</Typography.Text>;
  }

  return <Skeleton paragraph={false} active style={{ width: 100 }} />;
};
