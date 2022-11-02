import { useEffect, useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { TokenAmount } from '@jet-lab/margin';
import { BN } from '@project-serum/anchor';
import { Dictionary } from '@state/settings/localization/localization';
import { SwapsRowOrder } from '@state/views/views';
import { CurrentAccount } from '@state/user/accounts';
import { CurrentSwapOutput, TokenInputAmount, TokenInputString } from '@state/actions/actions';
import { CurrentSplSwapPool, SwapFees, SwapPoolTokenAmounts } from '@state/swap/splSwap';
import { CurrentPool } from '@state/pools/pools';
import { generateSwapPrices, getOutputTokenAmount } from '@utils/actions/swap';
import { useCurrencyFormatting } from '@utils/currency';
import { fromLocaleString } from '@utils/format';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import ApexCharts from 'apexcharts';
import { Typography } from 'antd';
import { LoadingOutlined } from '@ant-design/icons';

// Graph for displaying pricing and slippage data for current swap pair
export function SwapsGraph(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const currentAccount = useRecoilValue(CurrentAccount);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const [swapsRowOrder, setSwapsRowOrder] = useRecoilState(SwapsRowOrder);
  const currentPool = useRecoilValue(CurrentPool);
  const outputToken = useRecoilValue(CurrentSwapOutput);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const [tokenInputString, setTokenInputString] = useRecoilState(TokenInputString);
  const [currentChart, setCurrentChart] = useState<ApexCharts | undefined>(undefined);
  const swapPool = useRecoilValue(CurrentSplSwapPool);
  const swapPoolTokenAmounts = useRecoilValue(SwapPoolTokenAmounts);
  const swapFees = useRecoilValue(SwapFees);
  const swapPoolLoading = !swapPoolTokenAmounts;
  const swapSourceTokens = swapPoolTokenAmounts?.source.lamports.toNumber();
  const swapDestinationTokens = swapPoolTokenAmounts?.destination.lamports.toNumber();
  const swapMaxTradeAmount =
    currentAccount?.poolPositions[currentPool?.symbol ?? '']?.maxTradeAmounts.swap.lamports.toNumber();
  const [oraclePrice, setOraclePrice] = useState(0);
  const [poolPrice, setPoolPrice] = useState(0);
  const { Title, Text } = Typography;

  // Create and render chart on new data / market pair
  useEffect(() => {
    if (!swapPoolTokenAmounts || !swapPool) {
      return;
    }

    // Exponents
    const expoSource = Math.pow(10, swapPoolTokenAmounts.source.decimals);
    const expoDestination = Math.pow(10, swapPoolTokenAmounts.destination.decimals);
    // Get the swap pool account balances
    const balanceSourceToken = swapPoolTokenAmounts.source.lamports.toNumber();
    const balanceDestinationToken = swapPoolTokenAmounts.destination.lamports.toNumber();

    // Oracle price
    const oraclePrice = !swapPool.inverted
      ? currentPool!.tokenPrice / outputToken!.tokenPrice
      : outputToken!.tokenPrice / currentPool!.tokenPrice;
    setOraclePrice(oraclePrice);

    // Current pool price
    let poolPrice = 0.0;
    if (swapPool.pool.swapType === 'constantProduct') {
      const maybePrice = balanceDestinationToken / expoDestination / (balanceSourceToken / expoSource);
      poolPrice = !swapPool?.inverted ? maybePrice : 1.0 / maybePrice;
    } else if (swapPool.pool.swapType === 'stable') {
      poolPrice = oraclePrice;
    }
    setPoolPrice(poolPrice);
    const priceWithFee = !swapPool?.inverted ? 1 - swapFees : 1 + swapFees;

    // Show 2x what the user can trade
    const maxLeverage = (swapMaxTradeAmount ?? 0) * 2.0;

    // Destroy current chart if necessary
    const chartData: number[][] = generateSwapPrices(
      balanceSourceToken,
      balanceDestinationToken,
      maxLeverage,
      swapPool?.pool.swapType,
      swapFees,
      expoSource,
      expoDestination,
      !swapPool?.inverted,
      swapPool?.pool.amp ?? 1
    );
    currentChart?.destroy();
    if (!chartData) {
      setCurrentChart(undefined);
      return;
    }

    const worstOutput = chartData[chartData.length - 1][1];
    const range = Math.abs(poolPrice - worstOutput);

    // Quote token of the pool, uses Token B for consistency
    const poolQuoteToken = !swapPool.inverted ? ` ${outputToken?.symbol}` : ` ${currentPool?.symbol}`;

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
          click: function (event: any, chartContext?: any, config?: any) {
            try {
              const tokenAmount = new TokenAmount(
                new BN(config.config.series[config.seriesIndex].data[config.dataPointIndex][0] * expoSource),
                swapPoolTokenAmounts.source.decimals
              );
              setTokenInputString(tokenAmount.tokens.toString());
            } catch (e) {
              console.error(e);
            }
          }
        }
      },
      tooltip: {
        custom: function ({ series, seriesIndex, dataPointIndex, w }: any) {
          const xAmount = w.config.series[seriesIndex].data[dataPointIndex][0];
          const tokenAmount = new TokenAmount(new BN(xAmount * expoSource), swapPoolTokenAmounts.source.decimals);
          const outputAmount = getOutputTokenAmount(
            tokenAmount,
            swapPoolTokenAmounts.source,
            swapPoolTokenAmounts.destination,
            swapPool?.pool.swapType,
            swapFees,
            swapPool?.pool.amp ?? 1
          );
          const swapInString = currencyAbbrev((xAmount * expoSource) / expoSource);
          const swapOutString = currencyAbbrev(((outputAmount?.tokens ?? 0.0) * expoDestination) / expoDestination);
          const priceString = currencyAbbrev((series[seriesIndex][dataPointIndex] * 10000) / 10000);
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
            return currencyAbbrev(value, false, currentPool?.precision ?? 2);
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
            return currencyAbbrev(value, false, outputToken?.precision ?? 2);
          }
        },
        axisTicks: {
          show: true
        },
        show: true,
        // Bound price to 2% in either direction
        min: Math.round((poolPrice - range) * 99) / 100,
        max: Math.round((poolPrice + range) * 101) / 100,
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
    swapSourceTokens,
    swapDestinationTokens,
    swapPool,
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
    const output = getOutputTokenAmount(
      tokenInputAmount,
      swapPoolTokenAmounts?.source,
      swapPoolTokenAmounts?.destination,
      swapPool?.pool.swapType,
      swapFees,
      swapPool?.pool.amp ?? 1
    );
    if (!output) {
      return;
    }
    const swapPrice = !swapPool?.inverted
      ? output.tokens / tokenInputAmount.tokens
      : tokenInputAmount.tokens / output.tokens;

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
  }, [tokenInputString, swapPoolTokenAmounts]);

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
