import Title from 'antd/lib/typography/Title';
import { useRecoilState, useRecoilValue } from 'recoil';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Dictionary } from '@state/settings/localization/localization';
import { FixedBorrowRowOrder, FixedLendRowOrder } from '@state/views/fixed-term';
import { ResponsiveLineChart } from '@components/shared/charts/line-chart';
interface FixedChart {
  type: 'bids' | 'asks';
  decimals?: number;
}

export const FixedPriceChartContainer = ({ type }: FixedChart) => {
  const dictionary = useRecoilValue(Dictionary);
  const [rowOrder, setRowOrder] = useRecoilState(type === 'asks' ? FixedLendRowOrder : FixedBorrowRowOrder);

  return (
    <div className="fixed-term-graph view-element view-element-hidden flex align-center justify-end column">
      <div className="fixed-term-graph-head view-element-item view-element-item-hidden flex justify-center column">
        <div className="fixed-term-graph-head-info flex align-end">
          <div className="flex-centered">
            <Title level={2}>{dictionary.fixedView[type === 'asks' ? 'lend' : 'borrow'].chartTitle}</Title>
          </div>
        </div>
      </div>
      <ResponsiveLineChart />
      <ReorderArrows component="fixedChart" order={rowOrder} setOrder={setRowOrder} />
    </div>
  );
};
