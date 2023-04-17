import { useEffect, useMemo, useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { TokenAmount } from '@jet-lab/margin';
import { BN } from '@project-serum/anchor';
import { Dictionary } from '@state/settings/localization/localization';
import { SwapsRowOrder } from '@state/views/views';
import { CurrentAccount } from '@state/user/accounts';
import { CurrentSwapOutput, TokenInputAmount, TokenInputString } from '@state/actions/actions';
import { SwapFees } from '@state/swap/splSwap';
import { useCurrencyFormatting } from '@utils/currency';
import { fromLocaleString } from '@utils/format';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import ApexCharts from 'apexcharts';
import { Typography } from 'antd';
import { LoadingOutlined } from '@ant-design/icons';
import { useJetStore } from '@jet-lab/store';
import { Pools } from '@state/pools/pools';
import axios from 'axios';
import { SwapLiquidity } from '@utils/actions/swap';

// Graph for displaying pricing and slippage data for current swap pair
export function SwapsGraph(): JSX.Element {
  const { cluster } = useJetStore(state => state.settings);
  const dictionary = useRecoilValue(Dictionary);
  const currentAccount = useRecoilValue(CurrentAccount);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
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
  const [currentChart, setCurrentChart] = useState<ApexCharts | undefined>(undefined);
  const swapFees = useRecoilValue(SwapFees);
  const [chartData, setChartData] = useState<SwapLiquidity | undefined>(undefined);
  const swapPoolLoading = !chartData;
  const swapMaxTradeAmount =
    currentAccount?.poolPositions[currentPool?.symbol ?? '']?.maxTradeAmounts.swap.lamports.toNumber();
  const [oraclePrice, setOraclePrice] = useState(0);
  const [poolPrice, setPoolPrice] = useState(0);
  const { Title, Text } = Typography;
  const swapEndpoint = cluster === "mainnet-beta" ? "" : cluster === "devnet" ? process.env.REACT_APP_DEV_SWAP_API : process.env.REACT_APP_LOCAL_SWAP_API;


  // Create and render chart on new data / market pair
  useEffect(() => {
    if (!currentPool || !outputToken) {
      return;
    }

    // Exponents
    const expoSource = Math.pow(10, 3);
    const expoDestination = Math.pow(10, 3);

    // Oracle price
    const oraclePrice = !true
      ? currentPool!.tokenPrice / outputToken!.tokenPrice
      : outputToken!.tokenPrice / currentPool!.tokenPrice;
    setOraclePrice(oraclePrice);

    // Current pool price
    let poolPrice = 0.0;
    setPoolPrice(poolPrice);
    const priceWithFee = !true ? 1 - swapFees : 1 + swapFees;

    // Show 2x what the user can trade
    const maxLeverage = (swapMaxTradeAmount ?? 0) * 2.0;

    currentChart?.destroy();
    if (!chartData) {
      setCurrentChart(undefined);
      return;
    }

    // Quote token of the pool, uses Token B for consistency
    const poolQuoteToken = !true ? ` ${outputToken?.symbol}` : ` ${currentPool?.symbol}`;

    // Create and render new chart
    const swapsGraph = new ApexCharts(document.querySelector('.swaps-graph-container'), {
      chart: {
        type: 'line',
        width: '95%',
        height: '500px',
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
        },
        animations: {
          enabled: false
        },
        events: {
          click: function (_event: Event, _ctx?: any, config?: any) {
            // try {
            //   const tokenAmount = new TokenAmount(
            //     new BN(config.config.series[config.seriesIndex].data[config.dataPointIndex][0] * expoSource),
            //     swapPoolTokenAmounts.source.decimals
            //   );
            //   setTokenInputString(tokenAmount.tokens.toString());
            // } catch (e) {
            //   console.error(e);
            // }
          }
        }
      },
      tooltip: {
        custom: function ({ series, seriesIndex, dataPointIndex, w }: any) {
          const xAmount = w.config.series[seriesIndex].data[dataPointIndex][0];
          const tokenAmount = new TokenAmount(new BN(xAmount * expoSource), 6);
          // const outputAmount = getOutputTokenAmount(
          //   tokenAmount,
          //   swapPoolTokenAmounts.source,
          //   swapPoolTokenAmounts.destination,
          //   "constantProduct",
          //   swapFees,
          //   1
          // );
          const outputAmount = TokenAmount.zero(6);
          const swapInString = currencyAbbrev((xAmount * expoSource) / expoSource, currentPool?.precision ?? 2);
          const swapOutString = currencyAbbrev(
            ((outputAmount?.tokens ?? 0.0) * expoDestination) / expoDestination,
            currentPool?.precision ?? 2
          );
          const priceString = currencyAbbrev(
            (series[seriesIndex][dataPointIndex] * 10000) / 10000,
            currentPool?.precision ?? 2
          );
          return (
            '<div class="swaps-graph-tooltip">' +
            `<div class="flex align-center justify-between"><p>${dictionary.common.sell}</p> <p>${swapInString} ${
              currentPool?.symbol ?? ` ${dictionary.actions.swap.inputToken}`
            }</></div>` +
            `<div class="flex align-center justify-between"><p>${
              dictionary.actions.swap.receive
            }</p> <p>${swapOutString} ${outputToken?.symbol ?? ` ${dictionary.actions.swap.outputToken}`}</p></div>` +
            `<div class="flex align-center justify-between"><p>${dictionary.common.price}</p> <p>${priceString}${poolQuoteToken}</p></div>` +
            `<div style="flex-centered"><button class="small-btn">${dictionary.actions.swap.swapThis.toUpperCase()}</button></div>` +
            '</div>'
          );
        }
      },
      markers: {
        colors: ['var(--jet-blue)'],
        strokeWidth: 0
      },
      annotations: {
        yaxis: [
          {
            y: Math.round(poolPrice * priceWithFee * 10000) / 10000,
            y2: poolPrice,
            fillColor: 'var(--jet-green)',
            strokeWidth: '1px',
            borderWidth: '1px',
            opacity: 0.25
          },
          {
            id: 'pool-price',
            y: poolPrice,
            label: {
              text: `${dictionary.actions.swap.poolPrice}: ${currencyFormatter(poolPrice, false, 3)}${poolQuoteToken}`,
              offsetY: 6
            }
          },
          {
            id: 'oracle-price',
            y: oraclePrice,
            label: {
              text: `${dictionary.actions.swap.oraclePrice}: ${currencyFormatter(
                oraclePrice,
                false,
                3
              )}${poolQuoteToken}`,
              offsetY: 6
            }
          }
        ]
      },
      series: [
        {
          name: dictionary.actions.swap.swapsGraph.title,
          data: chartData
        }
      ],
      xaxis: {
        title: {
          text: `${dictionary.actions.swap.sellQuantity} (${currentPool?.symbol ?? '—'})`
        },
        labels: {
          padding: 0,
          formatter: (value: number) => {
            return currencyAbbrev(value, currentPool?.precision ?? 2, false);
          }
        },
        tooltip: {
          enabled: false
        },
        axisBorder: {
          show: true
        },
        axisTicks: {
          show: true
        },
        show: true,
        type: 'numeric',
        tickAmount: 5
      },
      yaxis: {
        title: {
          text: `${currentPool?.symbol ?? '—'} / ${outputToken?.symbol ?? '—'} ${dictionary.common.price}`
        },
        labels: {
          padding: 20,
          formatter: (value: number) => {
            return currencyAbbrev(value, outputToken?.precision ?? 2, false);
          }
        },
        axisTicks: {
          show: true
        },
        show: true,
        // Bound price to 2% in either direction
        // min: Math.round((poolPrice - range) * 99) / 100,
        // max: Math.round((poolPrice + range) * 101) / 100,
        tickAmount: 5
      },
      grid: {
        strokeDashArray: 5,
        xaxis: {
          lines: {
            show: true
          }
        },
        yaxis: {
          lines: {
            show: false
          }
        }
      },
      stroke: {
        width: [1],
        dashArray: [0],
        curve: 'straight'
      },
      plotOptions: {}
    });

    // Render chart
    swapsGraph.render();
    setCurrentChart(swapsGraph);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    // Only re-render the pool when the numbers in the pool change
    // The numbers will change when pools are changed (unless they're all empty)
    chartData,
    swapMaxTradeAmount
  ]);

  // Update chart annotations
  useEffect(() => {
    if (!currentChart) {
      return;
    }
    try {
      currentChart?.removeAnnotation('your-swap');
    } catch (e) {
      console.warn('Unable to remove any existing annotations', e);
    }
    if (tokenInputString === '0') {
      return;
    }
    // const output = getOutputTokenAmount(
    //   tokenInputAmount,
    //   swapPoolTokenAmounts?.source,
    //   swapPoolTokenAmounts?.destination,
    //   "constantProduct",
    //   swapFees,
    //   1
    // );
    // if (!output) {
    //   return;
    // }
    const output = TokenAmount.zero(6);
    const swapPrice = !true ? output.tokens / tokenInputAmount.tokens : tokenInputAmount.tokens / output.tokens;

    try {
      currentChart?.addPointAnnotation(
        {
          id: 'your-swap',
          x: parseFloat(fromLocaleString(tokenInputString)),
          y: swapPrice,
          marker: {
            size: 3
          },
          label: {
            text: `${dictionary.actions.swap.yourSwap}: ${tokenInputString} ${currentPool?.symbol ?? ''}`,
            offsetY: -2
          }
        },
        false
      );
    } catch (e) {
      console.warn('Unable to add annotations', e);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tokenInputString]);

  // Fetch chart data
  useEffect(() => {
    setChartData(undefined);
    if (!currentPool || !outputToken) {
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
      .get<SwapLiquidity>(`${swapEndpoint}/swap/liquidity/${from}/${to}/${maxAmount / fromExpo}`)
      .then(resp => {
        console.log('liquidity chart: ', resp.data);
        setChartData(resp.data);
      })
      .catch(err => err);
  }, [currentPool?.symbol, outputToken?.symbol, swapMaxTradeAmount]);

  return (
    <div className="swaps-graph view-element flex align-center justify-end column">
      <div className="swaps-graph-head flex justify-center column">
        <div className="swaps-graph-head-info flex flex-centered">
          <div className="flex-centered">
            <Title level={2}>{dictionary.actions.swap.swapsGraph.title}</Title>
          </div>
        </div>
      </div>
      <ConnectionFeedback />
      {swapPoolLoading && currentAccount && (
        <div className="overlay-message">
          <LoadingOutlined />
        </div>
      )}
      {currentChart && (
        <div className="swaps-graph-key flex-centered">
          <div className="swaps-graph-key-item flex-centered">
            <span className="swaps-graph-key-item-line price-impact"></span>
            <Text>{dictionary.actions.swap.priceImpact}</Text>
          </div>
          <div className="swaps-graph-key-item flex-centered">
            <span className="swaps-graph-key-item-line oracle-price"></span>
            <Text>{dictionary.actions.swap.oraclePrice}</Text>
          </div>
          <div className="swaps-graph-key-item flex-centered">
            <span className="swaps-graph-key-item-line pool-price"></span>
            <Text>{dictionary.actions.swap.poolPrice}</Text>
          </div>
          <div className="swaps-graph-key-item flex-centered">
            <span className={`swaps-graph-key-item-line swap-fees ${oraclePrice < poolPrice ? 'reverse' : ''}`}></span>
            <Text>{dictionary.actions.swap.swapFees}</Text>
          </div>
        </div>
      )}
      <div className="swaps-graph-container flex-centered"></div>
      <ReorderArrows component="swapsGraph" order={swapsRowOrder} setOrder={setSwapsRowOrder} />
    </div>
  );
}
