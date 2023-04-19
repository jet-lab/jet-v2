import { useEffect, useMemo, useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { SwapsRowOrder } from '@state/views/views';
import { CurrentAccount } from '@state/user/accounts';
import { CurrentSwapOutput, TokenInputAmount, TokenInputString } from '@state/actions/actions';
import { SwapFees } from '@state/swap/splSwap';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import { Typography } from 'antd';
import { useJetStore } from '@jet-lab/store';
import { Pools } from '@state/pools/pools';
import axios from 'axios';
import { SwapLiquidity } from '@utils/actions/swap';

import { Axis, LineSeries, Tooltip, XYChart } from '@visx/xychart';
import { scaleOrdinal } from '@visx/scale';
import { LegendItem, LegendLabel, LegendOrdinal } from '@visx/legend';

interface ISeries {
  id: string;
  data: Array<{ x: number; y: number }>;
  type: string;
}

interface ILineChart {
  width: number;
  height: number;
  paddingTop: number;
  paddingLeft: number;
  paddingRight: number;
  paddingBottom: number;
  series: ISeries[];
}

// Graph for displaying pricing and slippage data for current swap pair
export function SwapChart(): // ({
//   height,
//   width,
//   paddingTop,
//   paddingLeft,
//   paddingRight,
//   paddingBottom,
//   series
// }: ILineChart):
JSX.Element {
  const { cluster } = useJetStore(state => state.settings);
  const dictionary = useRecoilValue(Dictionary);
  const currentAccount = useRecoilValue(CurrentAccount);
  const [swapsRowOrder, setSwapsRowOrder] = useRecoilState(SwapsRowOrder);
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
  const swapFees = useRecoilValue(SwapFees);
  const [bidData, setBidData] = useState<{ x: number; y: number }[]>([
    { x: 0, y: 0 },
    { x: -1, y: -1 }
  ]);
  const [askData, setAskData] = useState<{ x: number; y: number }[]>([
    { x: 0, y: 0 },
    { x: -1, y: -1 }
  ]);
  const swapMaxTradeAmount =
    currentAccount?.poolPositions[currentPool?.symbol ?? '']?.maxTradeAmounts.swap.lamports.toNumber();
  const [oraclePrice, setOraclePrice] = useState(0);
  const [poolPrice, setPoolPrice] = useState(0);
  const { Title, Text } = Typography;
  const swapEndpoint =
    cluster === 'mainnet-beta'
      ? ''
      : cluster === 'devnet'
      ? process.env.REACT_APP_DEV_SWAP_API
      : process.env.REACT_APP_LOCAL_SWAP_API;

  // todo: create a function to generate these two data sets
  // y depends on the liquidity rage
  const dataOracle = [
    { x: oraclePrice, y: 0 },
    { x: oraclePrice, y: 250 }
  ];

  // once the scales are done, then pass in to LineSeries and Axis

  // legend
  const ordinalColorScale = scaleOrdinal({
    domain: ['Asks', 'Bids', 'Oracle Price'],
    range: ['#e36868', '#84c1ca', '#a79adb']
  });

  // Fetch chart data
  useEffect(() => {
    // setBidData([
    //   { x: 0, y: 0 },
    //   { x: 1, y: 1 }
    // ]);
    // setAskData([
    //   { x: 0, y: 0 },
    //   { x: -1, y: -1 }
    // ]);
    if (!currentPool || !outputToken) {
      console.log('11111');
      console.log('currentPool: ', currentPool);
      return;
    }
    // TODO: cache swapMaxTradeAmount and only rerun if significantly different
    const maxAmount = swapMaxTradeAmount;
    const from = currentPool.tokenMint.toString();
    const to = outputToken.tokenMint.toString();
    const fromExpo = Math.pow(10, currentPool.decimals);
    const toExpo = Math.pow(10, outputToken.decimals);
    if (!maxAmount) {
      return;
    }
    console.log('max swap amount: ', maxAmount / fromExpo);
    axios
      // .get<SwapLiquidity>(`${swapEndpoint}/swap/liquidity/${from}/${to}/${maxAmount / fromExpo}`)
      .get<SwapLiquidity>(
        `${swapEndpoint}/swap/liquidity/sBiatzH7agz9YScf12Jro16yrjrwB3UPLuSpfRo9kKY/GQUwhbmw9afvN7euH4Hofhobdz9EmjqrpaC3URRgX1KT/100`
      )
      // .get<SwapLiquidity>(
      //   `${swapEndpoint}/swap/liquidity/GQUwhbmw9afvN7euH4Hofhobdz9EmjqrpaC3URRgX1KT/6UEL83nioATpUWYUaqYSaDV9f1mhvxd7oxBUkdbqm8ZD/${
      //     maxAmount / fromExpo
      //   }`
      // )
      .then(resp => {
        console.log('liquidity chart: ', resp.data);
        // transform data

        setBidData(
          resp.data.bids.map(bid => {
            console.log('inside map bid data: ', bid);
            return {
              x: bid[0],
              y: bid[1]
            };
          })
        );
        console.log('bid data: ', bidData);
        setAskData(
          resp.data.asks.map(ask => {
            return {
              x: ask[0],
              y: ask[1]
            };
          })
        );
        console.log('ask data: ', askData);
      })
      .catch(err => err);
  }, [currentPool?.symbol, outputToken?.symbol, swapMaxTradeAmount]);

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
          dataKey="line2"
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
          dataKey="line3"
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

        {/* todo - set interface for the dataset */}
        {/* <LineSeries
          dataKey="line4"
          data={bids}
          xAccessor={d => d.x}
          yAccessor={d => d.y}
          stroke="#e36868"
          strokeWidth={2}
          strokeDasharray="3,3"
        />
        <LineSeries
          dataKey="line5"
          data={asks}
          xAccessor={d => d.x}
          yAccessor={d => d.y}
          stroke="#84c1ca"
          strokeWidth={2}
          strokeDasharray="3,3"
        /> */}

        <Axis
          key={`${dictionary.actions.swap.sellQuantity}`}
          label={`${dictionary.actions.swap.sellQuantity} (${currentPool?.symbol ?? '—'})`}
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
        <Axis
          key={`${currentPool?.symbol ?? '—'} / ${outputToken?.symbol ?? '—'} ${dictionary.common.price}`}
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
                {tooltipData &&
                  Object.entries(tooltipData.datumByKey).map(lineDataArray => {
                    const [key, value] = lineDataArray;

                    return (
                      <div className="row" key={key}>
                        <div className="dataType">{'price type:'}</div>
                        <div className="value">{'value'}</div>
                        {/* <div className="value">{yAccessor(value.datum)}</div> */}
                      </div>
                    );
                  })}
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
      {/* <ReorderArrows component="swapsGraph" order={swapsRowOrder} setOrder={setSwapsRowOrder} /> */}
    </div>
  );
}
