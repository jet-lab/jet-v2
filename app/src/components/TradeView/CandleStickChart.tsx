import { useEffect, useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { CurrentMarketPair } from '../../state/trade/market';
import {
  Period,
  periodOptions,
  CurrentPeriod,
  useCandles,
  PriceHistory,
  PriceHistoryLoading
} from '../../state/trade/priceHistory';
import { formatMarketPair, formatRate } from '../../utils/format';
import ApexCharts from 'apexcharts';
import { Dropdown, Menu, Skeleton, Typography } from 'antd';
import { TradeViewOrder } from '../../state/views/views';
import { ReorderArrows } from '../misc/ReorderArrows';
import { ReactComponent as ArrowUp } from '../../styles/icons/arrow-up.svg';
import { ReactComponent as ArrowDown } from '../../styles/icons/arrow-down.svg';
import { ReactComponent as AngleDown } from '../../styles/icons/arrow-angle-down.svg';
import { ReactComponent as TradeIcon } from '../../styles/icons/trade.svg';

export function CandleStickChart(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const [tradeViewOrder, setTradeViewOrder] = useRecoilState(TradeViewOrder);
  const currentMarketPair = useRecoilValue(CurrentMarketPair);
  const [currentPeriod, setCurrentPeriod] = useRecoilState(CurrentPeriod);
  const pairPriceHistory = useRecoilValue(PriceHistory);
  const priceHistory = pairPriceHistory[currentPeriod];
  const priceHistoryLoading = useRecoilValue(PriceHistoryLoading);
  const candles = useCandles(currentPeriod);
  const [currentChart, setCurrentChart] = useState<ApexCharts | undefined>(undefined);
  const { Title, Paragraph, Text } = Typography;

  // 9 Skeleton lines for loading animation
  const skeletons = (): JSX.Element[] => {
    const skeletons: JSX.Element[] = [];
    for (let i = 0; i < 9; i++) {
      skeletons.push(
        <Skeleton
          key={i}
          className="align-center view-element-item"
          paragraph={false}
          active={!candles && priceHistoryLoading}
        />
      );
    }
    return skeletons;
  };

  // Create and render chart on new data / market pair
  useEffect(() => {
    // Destroy current chart if necessary
    currentChart?.destroy();
    if (!candles) {
      setCurrentChart(undefined);
      return;
    }

    // Parse candles into usable data structure for chart
    const chartData: { x: Date; y: number[] }[] = [];
    let yMin = candles[candles.length - 1] ? candles[candles.length - 1][3] : 0;
    let yMax = candles[candles.length - 1] ? candles[candles.length - 1][2] : 0;
    for (let i = 0; i < candles.length; i++) {
      chartData.push({
        x: new Date(candles[i][0] * 1000),
        y: candles[i].slice(1, candles[i].length - 1)
      });
      // Min for Y axis formatting
      if (candles[i][3] < yMin) {
        yMin = candles[i][3];
      }
      // Max for Y axis formatting
      if (candles[i][2] > yMax) {
        yMax = candles[i][2];
      }
    }

    // Create and render new chart
    const candleStickChart = new ApexCharts(document.querySelector('.candle-stick-chart-container'), {
      chart: {
        type: 'candlestick',
        width: '100%',
        height: '350px',
        zoom: {
          zoomedArea: {
            fill: {
              color: 'var(--jet-green-3)',
              opacity: 0.4
            },
            stroke: {
              color: 'var(--jet-green-3)',
              opacity: 0.8,
              width: 1
            }
          }
        }
      },
      series: [{ data: chartData }],
      xaxis: { type: 'datetime' },
      yaxis: {
        show: false,
        tooltip: { enabled: true },
        min: yMin - yMin / 100,
        max: yMax + yMax / 100
      },
      grid: {
        xaxis: {
          lines: { show: true }
        },
        yaxis: {
          lines: { show: true }
        }
      },
      plotOptions: {
        candlestick: {
          colors: {
            upward: 'var(--success)',
            downward: 'var(--danger)'
          },
          wick: { useFillColor: true }
        }
      }
    });
    candleStickChart.render();
    setCurrentChart(candleStickChart);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentMarketPair, candles]);

  return (
    <div className="candle-stick-chart view-element view-element-hidden flex align-center justify-end column">
      <div className="candle-stick-chart-head view-element-item view-element-item-hidden flex justify-center column">
        <div className="candle-stick-chart-head-info flex align-end">
          <div className="flex-centered">
            <TradeIcon className="jet-icon trade-icon" />
            <Title level={2}>{formatMarketPair(currentMarketPair)}</Title>
            {priceHistory && <Text>&bull;</Text>}
            <Title level={2}>
              {priceHistory
                ? /* @ts-ignore */
                  dictionary.tradeView.candleStickChart.period[
                    `${periodOptions[currentPeriod].timeframeIndexer}Timeframe`
                  ]
                : ''}
            </Title>
            {priceHistory && <Text>&bull;</Text>}
            {priceHistory && priceHistory.percentageChange ? (
              <Title level={2} className={priceHistory.percentageChange > 0 ? 'price-up' : 'price-down'} italic>
                {priceHistory ? (
                  priceHistory.percentageChange > 0 ? (
                    <ArrowUp className="jet-icon" />
                  ) : (
                    <ArrowDown className="jet-icon" />
                  )
                ) : (
                  <></>
                )}
                {priceHistory ? formatRate(priceHistory.percentageChange).replaceAll('-', '') : ''}
              </Title>
            ) : (
              <></>
            )}
          </div>
          <div className="candle-stick-chart-head-period flex-centered">
            <Text italic>{dictionary.tradeView.candleStickChart.period.title}:</Text>
            <Dropdown
              overlay={
                <Menu>
                  {Object.keys(periodOptions).map(period => (
                    <Menu.Item
                      key={period}
                      onClick={() => setCurrentPeriod(period as Period)}
                      className={period === currentPeriod ? 'active' : ''}>
                      {/* @ts-ignore */}
                      {dictionary.tradeView.candleStickChart.period[periodOptions[period].timeframeIndexer]}
                    </Menu.Item>
                  ))}
                </Menu>
              }>
              <Text type="secondary">
                {/* @ts-ignore */}
                {dictionary.tradeView.candleStickChart.period[periodOptions[currentPeriod].timeframeIndexer]}
                <AngleDown className="jet-icon" />
              </Text>
            </Dropdown>
          </div>
        </div>
        <Paragraph className="candle-stick-chart-head-subinfo" italic>
          {dictionary.tradeView.candleStickChart.exchangeDisclaimer}
        </Paragraph>
      </div>
      <div className="candle-stick-chart-container view-element-item view-element-item-hidden flex-centered"></div>
      {!candles && <div className="loading-skeletons flex-centered column">{skeletons()}</div>}
      <ReorderArrows component="candleStickChart" order={tradeViewOrder} setOrder={setTradeViewOrder} vertical />
    </div>
  );
}
