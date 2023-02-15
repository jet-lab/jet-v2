import { useCurrencyFormatting } from '@utils/currency';
import { Skeleton, Typography } from 'antd';
import { WithPoolData } from './PoolDetail';

// Renders the total borrowed to accompany the pie chart
export const TotalBorrowed = ({ pool }: WithPoolData) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  let render = <Skeleton paragraph={false} active style={{ marginTop: 5 }} />;
  if (pool) {
    const borrowedAbbrev = currencyAbbrev(pool.borrowed_tokens, pool.precision, false, undefined);
    render = (
      <div className="pie-chart-section-info-item">
        <Typography.Text type="danger">{borrowedAbbrev}</Typography.Text>
      </div>
    );
  }

  return render;
};
