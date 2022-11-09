import Title from 'antd/lib/typography/Title';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Order } from '@jet-lab/jet-bonds-client';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Dictionary } from '@state/settings/localization/localization';
import { FixedBorrowRowOrder, FixedLendRowOrder } from '@state/views/fixed-term';
import { ExtendedOrderBook } from '@state/fixed-market/fixed-term-market-sync';
import { ResponsiveLineChart } from '@components/shared/charts/line-chart';

interface Formatter {
  currencyFormatter: (
    value: number,
    fiatValues?: boolean | undefined,
    decimals?: number | undefined,
    ceil?: boolean | undefined
  ) => string;
  currencyAbbrev: (
    total: number,
    fiatValues?: boolean | undefined,
    price?: number | undefined,
    decimals?: number | undefined
  ) => string;
}

interface FixedChart {
  type: 'bids' | 'asks';
  decimals?: number;
}

interface DataPoint {
  x: number;
  y: number;
}

// Setup data for the chart
const getChartData = (orders: Order[]): DataPoint[] =>
  orders.reduce((all, order) => {
    const previousPoint = all.length > 0 ? all[all.length - 1] : { x: 0, y: 0 };
    const point = {
      x: previousPoint.x + Number(order.quote_size),
      y: previousPoint.y + Number(order.base_size)
    };
    if (typeof point.x === 'number' && typeof point.y === 'number') all.push(point);
    return all;
  }, [] as Array<{ x: number; y: number }>);

const getOptions = (books: ExtendedOrderBook[], decimals: number, type: string, formatting: Formatter) => {
  const series = books.map(book => ({
    name: book.name,
    data: getChartData(type === 'asks' ? book.asks : book.bids)
  }));

  return {
    chart: {
      type: 'line',
      width: '100%',
      height: '515px'
    },
    xaxis: {
      type: 'numeric',
      tickAmount: 10,
      labels: {
        formatter: (val: string) => formatting.currencyFormatter(parseInt(val), false, decimals)
      }
    },
    yaxis: {
      type: 'numeric',
      tickAmount: 10,
      labels: {
        formatter: (val: string) => formatting.currencyFormatter(parseInt(val), false, decimals)
      }
    },
    series,
    theme: {
      mode: 'dark'
    },
    tooltip: {
      enabled: true,
      custom: ({ dataPointIndex, seriesIndex }: any) => {
        const item = series[seriesIndex].data[dataPointIndex];
        if (!item) return;
        const rate = 1 / (item.x / item.y) - 1; // TODO FIXME, scale rate by tenor length
        const amount = item.x;
        const repay = item.y;
        return `<div className="flex-centered column">
            <div>Amount ${formatting.currencyFormatter(amount, false, 2)}</div>
            <div>Repay ${formatting.currencyFormatter(repay, false, 2)}</div>
            <div>Rate ${(rate * 100).toFixed(2)}%</div>
          </div>`;
      }
    }
  };
};

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
