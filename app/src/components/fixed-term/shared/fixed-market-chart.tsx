import Title from 'antd/lib/typography/Title';
import { useRecoilState, useRecoilValue } from 'recoil';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { FixedBorrowRowOrder, FixedLendRowOrder } from '@state/views/fixed-term';
import { ResponsiveLineChart } from '@components/fixed-term/shared/charts/line-chart';
import {
  CurrentOrderTab,
  CurrentOrderTabAtom,
  FixedMarketAtom,
  MarketAndconfig
} from '@state/fixed-market/fixed-term-market-sync';
import { friendlyMarketName } from '@utils/jet/fixed-term-utils';
interface FixedChart {
  type: 'bids' | 'asks';
  decimals?: number;
}

const getChartTitle = (currentTab: CurrentOrderTab, market: MarketAndconfig) => {
  if (!market?.config?.symbol) return '';
  switch (currentTab) {
    case 'borrow-now':
      return `${friendlyMarketName(market.config.symbol, market.config.borrowDuration)} loan offers`;
    case 'lend-now':
      return `${friendlyMarketName(market.config.symbol, market.config.borrowDuration)} borrow requests`;
    case 'offer-loan':
      return `${market.config.symbol} loan offers`;
    case 'request-loan':
      return `${market.config.symbol} borrow requests`;
  }
};

export const FixedPriceChartContainer = ({ type }: FixedChart) => {
  const [rowOrder, setRowOrder] = useRecoilState(type === 'asks' ? FixedLendRowOrder : FixedBorrowRowOrder);
  const currentTab = useRecoilValue(CurrentOrderTabAtom);
  const market = useRecoilValue(FixedMarketAtom);

  return (
    <div className="fixed-term-graph view-element view-element-hidden flex align-center justify-end column">
      <div className="fixed-term-graph-head view-element-item view-element-item-hidden flex justify-center column">
        <div className="fixed-term-graph-head-info flex align-end">
          <div className="flex-centered">
            <Title level={2}>{getChartTitle(currentTab, market)}</Title>
          </div>
        </div>
      </div>
      <ResponsiveLineChart />
      <ReorderArrows component="fixedChart" order={rowOrder} setOrder={setRowOrder} />
    </div>
  );
};
