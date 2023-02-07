import { WithPoolData } from './PoolDetail';
import { useCurrencyFormatting } from '@utils/currency';
import { Skeleton, Typography } from 'antd';

// Renders the available liquidity to accompany the pie chart
export const AvailableLiquidity = ({ pool }: WithPoolData) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  let render = <Skeleton paragraph={false} active style={{ marginTop: 5 }} />;
  if (pool) {
    const vaultAbbrev = currencyAbbrev(pool.deposit_tokens - pool.borrowed_tokens, pool.precision, false, undefined);
    render = (
      <div className="pie-chart-section-info-item">
        <Typography.Text type="success">{vaultAbbrev}</Typography.Text>
      </div>
    );
  }

  return render;
};
