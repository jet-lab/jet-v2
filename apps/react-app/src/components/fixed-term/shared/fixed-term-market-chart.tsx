import Title from 'antd/lib/typography/Title';
import { useRecoilState, useRecoilValue } from 'recoil';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { FixedBorrowRowOrder, FixedLendRowOrder } from '@state/views/fixed-term';
import { DepthChart } from '@components/charts/depth-chart';
import {
  AllFixedTermMarketsOrderBooksAtom,
  CurrentOrderTabAtom,
  FixedTermMarketAtom,
  SelectedFixedTermMarketAtom
} from '@state/fixed-term/fixed-term-market-sync';
import { friendlyMarketName } from '@utils/jet/fixed-term-utils';
import { Suspense, useMemo } from 'react';
import { MainConfig } from '@state/config/marginConfig';
import { MarginTokenConfig, TokenAmount, bigIntToBn } from '@jet-lab/margin';
import { LoadingOutlined } from '@ant-design/icons';
import { ParentSizeModern } from '@visx/responsive';
import { LegendItem, LegendLabel, LegendOrdinal } from '@visx/legend';
import { scaleOrdinal } from '@visx/scale';

const ordinalColorScale = scaleOrdinal({
  domain: ['Loan requests', 'Loan offers'],
  range: ['#e36868', '#84c1ca']
});

interface FixedChart {
  type: 'bids' | 'asks';
}

const FixedTermChart = ({ currentTab, token }: { currentTab: string; token: MarginTokenConfig }) => {
  const selectedMarketIndex = useRecoilValue(SelectedFixedTermMarketAtom);
  const openOrders = useRecoilValue(AllFixedTermMarketsOrderBooksAtom);

  const { asksAscending, bidsDescending, xRange, yRange } = useMemo(() => {
    const marketData = openOrders[selectedMarketIndex];
    const liquidity = marketData.orderbook.sampleLiquidity(1000000000000000n);
    const bids = liquidity.asks.map(x => [x[0], new TokenAmount(bigIntToBn(x[1]), token.decimals).tokens]) as [
      price: number,
      amt: number
    ][];

    if (bids.length > 0) {
      bids.unshift([0, bids[0][1]]);
    }
    return {
      asksAscending: liquidity.bids.map(x => [x[0], new TokenAmount(bigIntToBn(x[1]), token.decimals).tokens]) as [
        price: number,
        amt: number
      ][],
      bidsDescending: liquidity.asks.map(x => [x[0], new TokenAmount(bigIntToBn(x[1]), token.decimals).tokens]) as [
        price: number,
        amt: number
      ][],
      xRange: [liquidity.price_range[0], liquidity.price_range[1]] as [min: number, max: number],
      yRange: [0, new TokenAmount(bigIntToBn(liquidity.liquidity_range[1]), token.decimals).tokens] as [
        min: number,
        max: number
      ]
    };
  }, [openOrders, currentTab, selectedMarketIndex]);

  if (!token) return <LoadingOutlined />;

  return (
    <ParentSizeModern>
      {({ height, width }) => (
        <DepthChart
          asksAscending={asksAscending}
          bidsDescending={bidsDescending}
          asksColor="#84c1ca"
          bidsColor="#e36868"
          height={height}
          width={width}
          xRange={xRange}
          yRange={yRange}
          xLabel="Annualized Rate"
          yLabel={`${token.symbol} amount`}
          base={{ symbol: token.symbol, expo: -token.decimals }}
          quote={{ symbol: token.symbol, expo: -token.decimals }}
          isPct={true}
        />
      )}
    </ParentSizeModern>
  );
};

export const FixedPriceChartContainer = ({ type }: FixedChart) => {
  const [rowOrder, setRowOrder] = useRecoilState(type === 'asks' ? FixedLendRowOrder : FixedBorrowRowOrder);
  const currentTab = useRecoilValue(CurrentOrderTabAtom);

  const market = useRecoilValue(FixedTermMarketAtom);

  const marginConfig = useRecoilValue(MainConfig);

  const token = useMemo(() => {
    if (!marginConfig || !market) return null;
    return Object.values(marginConfig?.tokens).find(token => {
      return market.config.underlyingTokenMint === token.mint.toString();
    });
  }, [marginConfig, market?.config]);

  if (!market) return null;

  return (
    <div className="fixed-term-graph view-element view-element-hidden align-center column flex justify-end">
      <div className="fixed-term-graph-head view-element-item view-element-item-hidden column flex justify-center">
        <div className="fixed-term-graph-head-info align-end flex">
          <div className="flex-centered">
            <Title level={2}>{`${friendlyMarketName(
              market.config.symbol,
              market.config.borrowTenor
            )} open orders`}</Title>
          </div>
        </div>
      </div>
      {market && (
        <Suspense>
          <div className="swaps-chart-root">
            {!market || !currentTab || !token ? (
              <LoadingOutlined />
            ) : (
              <>
                <div className="swaps-chart-legend">
                  <LegendOrdinal scale={ordinalColorScale} labelFormat={label => `${label.toUpperCase()}`}>
                    {labels => (
                      <div style={{ display: 'flex', flexDirection: 'row' }}>
                        {labels.map((label, i) => (
                          <LegendItem key={`legend-quantile-${i}`} margin="0 5px">
                            <svg width={15} height={15}>
                              <rect fill={label.value} width={15} height={15} />
                            </svg>
                            <LegendLabel align="left" margin="0 0 0 4px">
                              {label.text}
                            </LegendLabel>
                          </LegendItem>
                        ))}
                      </div>
                    )}
                  </LegendOrdinal>
                </div>
                <FixedTermChart currentTab={currentTab} token={token} />
              </>
            )}
          </div>
        </Suspense>
      )}
      <ReorderArrows component="fixedChart" order={rowOrder} setOrder={setRowOrder} />
    </div>
  );
};
