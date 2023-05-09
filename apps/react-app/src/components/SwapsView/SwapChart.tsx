import { useEffect, useMemo, useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { CurrentAccount } from '@state/user/accounts';
import { CurrentSwapOutput, TokenInputAmount, TokenInputString } from '@state/actions/actions';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import { Typography } from 'antd';
import { useJetStore, getSwapLiquidity } from '@jet-lab/store';
import { Pools } from '@state/pools/pools';

import { Axis, LineSeries, Tooltip, XYChart } from '@visx/xychart';
import { scaleLinear, scaleOrdinal } from '@visx/scale';
import { LegendItem, LegendLabel, LegendOrdinal } from '@visx/legend';

// Graph for displaying pricing and slippage data for current swap pair
export function SwapChart(): JSX.Element {
  const { cluster } = useJetStore(state => state.settings);
  const dictionary = useRecoilValue(Dictionary);
  const currentAccount = useRecoilValue(CurrentAccount);
  const selectedPoolKey = useJetStore(state => state.selectedPoolKey);
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );
  const outputToken = useRecoilValue(CurrentSwapOutput);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const [tokenInputString, setTokenInputString] = useRecoilState(TokenInputString);
  const [quoteToken, setQuoteToken] = useState<string>('-');
  const [bidData, setBidData] = useState<{ x: number; y: number }[]>([
    { x: 0, y: 0 },
    { x: -1, y: -1 }
  ]);
  const [askData, setAskData] = useState<{ x: number; y: number }[]>([
    { x: 0, y: 0 },
    { x: -1, y: -1 }
  ]);
  const [priceRange, setPriceRange] = useState<[number, number]>([0, 0]);
  const [liquidityRange, setLiquidityRange] = useState<[number, number]>([0, 0]);
  const swapMaxTradeAmount =
    currentAccount?.poolPositions[currentPool?.symbol ?? '']?.maxTradeAmounts.swap.lamports.toNumber();
  const { Title } = Typography;
  const swapEndpoint: string =
    cluster === 'mainnet-beta'
      ? ''
      : cluster === 'devnet'
      ? String(process.env.REACT_APP_DEV_SWAP_API)
      : String(process.env.REACT_APP_LOCAL_SWAP_API);

  const amount = useMemo(() => {
    if (swapMaxTradeAmount && currentPool && outputToken) {
      const fromExpo = Math.pow(10, currentPool.decimals);
      return swapMaxTradeAmount / fromExpo;
    }
    return 0;
  }, [swapMaxTradeAmount, currentPool, outputToken]);

  const { data, isLoading, error } = getSwapLiquidity(
    swapEndpoint,
    currentPool?.tokenMint.toString(),
    outputToken?.tokenMint.toString(),
    amount
  );
  // legend
  const ordinalColorScale = scaleOrdinal({
    domain: ['Asks', 'Bids', 'Oracle Price'],
    range: ['#e36868', '#84c1ca', '#a79adb']
  });

  const xScale = useMemo(() => {
    return scaleLinear<number>({
      domain: [priceRange[0], priceRange[1]],
      range: [priceRange[0], priceRange[1]],
      clamp: true
    });
  }, [priceRange]);

  const yScale = useMemo(() => {
    return scaleLinear<number>({
      domain: [0, liquidityRange[1]],
      range: [0, liquidityRange[1]],
      clamp: true
    });
  }, [liquidityRange]);

  // Fetch chart data
  useEffect(() => {
    if (!data) {
      return;
    }

    // set quote token
    setQuoteToken(data.quote);

    // set the xy boundaries
    setPriceRange([data.price_range[0], data.price_range[1]]);
    setLiquidityRange([data.liquidity_range[0], data.liquidity_range[1]]);

    // transform data
    setBidData(
      data.bids.map(bid => {
        return {
          x: bid[0],
          y: bid[1]
        };
      })
    );
    setAskData(
      data.asks.map(ask => {
        return {
          x: ask[0],
          y: ask[1]
        };
      })
    );
  }, [data]);

  // Oracle price
  const oraclePrice = useMemo(() => {
    if (currentPool?.tokenPrice && outputToken?.tokenPrice) {
      return currentPool?.tokenMint.toString() === quoteToken
        ? outputToken!.tokenPrice / currentPool!.tokenPrice
        : currentPool!.tokenPrice / outputToken!.tokenPrice;
    } else {
      return 0;
    }
  }, [currentPool, outputToken, quoteToken]);

  const dataOracle = [
    { x: oraclePrice, y: liquidityRange[0] },
    { x: oraclePrice, y: liquidityRange[1] }
  ];

  return (
    <div className="swaps-graph view-element align-center column flex justify-end">
      <div className="swaps-graph-head column flex justify-center">
        <div className="swaps-graph-head-info flex-centered flex">
          <div className="flex-centered">
            <Title level={2}>{dictionary.actions.swap.swapsGraph.title}</Title>
          </div>
        </div>
      </div>
      <ConnectionFeedback />
      {/* {swapPoolLoading && currentAccount && (
        <div className="overlay-message">
          <LoadingOutlined />
        </div>
      )} */}
      <div className="swaps-graph-container flex-centered"></div>

      <LegendOrdinal scale={ordinalColorScale} labelFormat={label => `${label.toUpperCase()}`}>
        {labels => (
          <div style={{ display: 'flex', flexDirection: 'row', marginTop: 500 }}>
            {labels.map((label, i) => (
              <LegendItem
                key={`legend-quantile-${i}`}
                margin="0 5px"
                onClick={() => {
                  alert(`clicked: ${JSON.stringify(label)}`);
                }}>
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

      <XYChart
        height={500}
        margin={{ left: 80, top: 50, bottom: 50, right: 80 }}
        xScale={{ type: 'linear' }}
        yScale={{ type: 'linear' }}>
        <LineSeries
          dataKey="bids"
          data={bidData}
          xAccessor={d => {
            if (d === undefined) {
              return 0;
            }
            return d.x;
          }}
          yAccessor={d => {
            if (d === undefined) {
              return 0;
            }
            return d.y;
          }}
          stroke="#84c1ca"
          strokeWidth={2}
          strokeDasharray="3,3"
        />
        <LineSeries
          dataKey="asks"
          data={askData}
          xAccessor={d => {
            if (d === undefined) {
              return 0;
            }
            return d.x;
          }}
          yAccessor={d => {
            if (d === undefined) {
              return 0;
            }
            return d.y;
          }}
          stroke="#e36868"
          strokeWidth={2}
          strokeDasharray="3,3"
        />

        <Axis
          key={dictionary.actions.swap.sellQuantity}
          label={dictionary.actions.swap.sellQuantity}
          orientation="left"
          left={80}
          numTicks={5}
          labelProps={{ fill: 'rgb(199, 199, 199)', fontSize: 12, textAnchor: 'middle' }}
          tickLabelProps={() => ({
            fontSize: 10,
            fill: '#fff',
            opacity: 0.6,
            textAnchor: 'middle',
            dy: 8
          })}
          // todo - fix: scale = {yScale} modify to use AxisLeft
        />
        {/* <AxisLeft
          key={dictionary.actions.swap.sellQuantity}
          label={dictionary.actions.swap.sellQuantity}
          scale={yScale}
          left={80}
          numTicks={5}
          labelProps={{ fill: 'rgb(199, 199, 199)', fontSize: 12, textAnchor: 'middle' }}
          tickLabelProps={() => ({
            fontSize: 10,
            fill: '#fff',
            opacity: 0.6,
            textAnchor: 'middle',
            dy: 8
          })}
        /> */}
        <Axis
          key={`baseQuote[0] / ${outputToken?.symbol ?? '—'} ${dictionary.common.price}`}
          label={`${currentPool?.symbol ?? '—'} / ${outputToken?.symbol ?? '—'} ${dictionary.common.price}`}
          orientation="bottom"
          top={450}
          numTicks={5}
          labelProps={{ fill: 'rgb(199, 199, 199)', fontSize: 12, dx: -25, textAnchor: 'middle' }}
          tickLabelProps={() => ({
            fontSize: 10,
            fill: '#fff',
            opacity: 0.6,
            textAnchor: 'end',
            dy: 4,
            dx: -8
          })}
          // todo - fix: scale = {xScale} modify to use AxisBottom
        />

        <Tooltip
          snapTooltipToDatumX
          snapTooltipToDatumY
          showSeriesGlyphs
          glyphStyle={{
            fill: '#008561',
            strokeWidth: 0
          }}
          renderTooltip={({ tooltipData }) => {
            return (
              <>
                {tooltipData && (
                  <div className="swaps-graph-tooltip">
                    <div className="align-center flex justify-between">
                      <p>{dictionary.common.sell}</p>
                      <p>{`# ${currentPool?.symbol ?? dictionary.actions.swap.inputToken}`}</p>
                    </div>

                    <div className="align-center flex justify-between">
                      <p>{dictionary.actions.swap.receive}</p>
                      <p>{`# ${outputToken?.symbol ?? dictionary.actions.swap.outputToken}`}</p>
                    </div>
                    <div className="align-center flex justify-between">
                      <p>{dictionary.common.price}</p>
                      <p>{`$$$ ${outputToken?.symbol ?? dictionary.actions.swap.outputToken}`}</p>
                    </div>
                  </div>
                )}
              </>
            );
          }}
        />
        {/* todo: fix - oracle line should be center of the graph */}
        <LineSeries
          stroke="#a79adb"
          dataKey="oracle-price"
          data={dataOracle}
          xAccessor={d => {
            if (d === undefined) {
              return 0;
            }
            return d.x;
          }}
          yAccessor={d => {
            if (d === undefined) {
              return 0;
            }
            return d.y;
          }}
          // xAccessor={d => xScale(d.x) || 0}
          // yAccessor={d => yScale(d.y) || 0}
        />
      </XYChart>
    </div>
  );
}

// todo - fix x y scale - currently undefined
// todo - set bottow axis quote priority: usdc > usdt > sol
// todo - fix tooltip info
// todo - fix output token state for swap entry
// todo - fix entry state error, not match update, one state behind
