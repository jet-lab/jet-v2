import { WithPoolData } from './PoolDetail';
import { useCurrencyFormatting } from '@utils/currency';
import { Skeleton, Typography } from 'antd';

// Renders the available liquidity to accompany the pie chart
export const AvailableLiquidity = ({ selectedPool }: WithPoolData) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  let render = <Skeleton paragraph={false} active style={{ marginTop: 5 }} />;
  if (selectedPool) {
    const vaultAbbrev = currencyAbbrev(
      selectedPool.deposit_tokens - selectedPool.borrowed_tokens,
      selectedPool.precision,
      false,
      undefined
    );
    render = (
      <div className="pie-chart-section-info-item">
        <Typography.Text type="success">{vaultAbbrev}</Typography.Text>
      </div>
    );
  }

  return render;
};
