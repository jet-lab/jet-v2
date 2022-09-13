import { useEffect, useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import ApexCharts from 'apexcharts';
import { Typography } from 'antd';
import { SwapsRowOrder } from '../../state/views/views';
import { ReorderArrows } from '../misc/ReorderArrows';
import { CurrentSwapOutput, TokenInputAmount, TokenInputString } from '../../state/actions/actions';
import { CurrentPool } from '../../state/pools/pools';

import { generateSwapPrices, getOutputTokenAmount } from '../../utils/actions/swap';
import { useCurrencyFormatting } from '../../utils/currency';
import { TokenAmount } from '@jet-lab/margin';
import { BN } from '@project-serum/anchor';
import { CurrentAccount } from '../../state/user/accounts';
import { LoadingOutlined } from '@ant-design/icons';
import { CurrentSplSwapPool, SwapFees, SwapPoolTokenAmounts } from '../../state/swap/splSwap';

// TODO: clean and match the way the other component files are structured
export function SwapsGraph(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const currentAccount = useRecoilValue(CurrentAccount);
  const { currencyFormatter } = useCurrencyFormatting();
  const [swapsRowOrder, setSwapsRowOrder] = useRecoilState(SwapsRowOrder);
  const currentPool = useRecoilValue(CurrentPool);
  const outputToken = useRecoilValue(CurrentSwapOutput);
  // For updating token value on chart interaction
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const [tokenInputString, setTokenInputString] = useRecoilState(TokenInputString);
  const [currentChart, setCurrentChart] = useState<ApexCharts | undefined>(undefined);
  const { Title } = Typography;
  const swapPool = useRecoilValue(CurrentSplSwapPool);
  const swapPoolTokenAmounts = useRecoilValue(SwapPoolTokenAmounts);
  const swapFees = useRecoilValue(SwapFees);
  const loading = !swapPoolTokenAmounts;
  const swapSourceTokens = swapPoolTokenAmounts?.source.lamports.toNumber();
  const swapDestinationTokens = swapPoolTokenAmounts?.destination.lamports.toNumber();
  const swapMaxTradeAmount =
    currentAccount?.poolPositions[currentPool?.symbol ?? '']?.maxTradeAmounts.swap.lamports.toNumber();

  // Create and render chart on new data / market pair
  useEffect(() => {
    if (!swapPoolTokenAmounts || !currentAccount || !swapPool) {
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

    // Current pool price
    let poolPrice = 0.0;
    if (swapPool.pool.swapType === 'constantProduct') {
      const maybePrice = balanceDestinationToken / expoDestination / (balanceSourceToken / expoSource);
      poolPrice = !swapPool?.inverted ? maybePrice : 1.0 / maybePrice;
    } else if (swapPool.pool.swapType === 'stable') {
      poolPrice = oraclePrice;
    }
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

    // Price annotation prefix and suffix
    const priceAnnotationPrefix = swapPool.pool.swapType === 'constantProduct' ? '$' : '';
    const swapPoolKey = swapPool.inverted
      ? `${outputToken?.symbol}/${currentPool?.symbol}`
      : `${currentPool?.symbol}/${outputToken?.symbol}`;
    const priceAnnotationSuffix = swapPool.pool.swapType === 'constantProduct' ? '' : ` ${swapPoolKey}`;

    // Create and render new chart
    const swapsGraph = new ApexCharts(document.querySelector('.swaps-graph-container'), {
      chart: {
        type: 'line',
        width: '98%',
        height: '530px',
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
        toolbar: {
          show: true
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
              setTokenInputString(tokenAmount.uiTokens);
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
          const swapInString = Math.round(xAmount * expoSource) / expoSource;
          const swapOutString = Math.round((outputAmount?.tokens ?? 0.0) * expoDestination) / expoDestination;
          const priceString = Math.round(series[seriesIndex][dataPointIndex] * 10000) / 10000;
          return (
            '<div class="swaps-graph-tooltip">' +
            `<p>${dictionary.common.sell}: <strong>${swapInString} ${
              currentPool?.symbol ?? ` ${dictionary.actions.swap.inputToken}`
            }</strong></p>` +
            `<p>${dictionary.actions.swap.recieve}: <strong>${swapOutString} ${
              outputToken?.symbol ?? ` ${dictionary.actions.swap.outputToken}`
            }</strong></p>` +
            `<p>${dictionary.common.price}: <strong>${priceAnnotationPrefix}${priceString}${priceAnnotationSuffix}</strong>` +
            `<div style='flex flex-centered'><button class='small-btn'>${dictionary.actions.swap.swapThis.toUpperCase()}</button></div>` +
            '</div>'
          );
        }
      },
      annotations: {
        yaxis: [
          {
            y: Math.round(poolPrice * priceWithFee * 10000) / 10000,
            y2: poolPrice,
            fillColor: 'var(--jet-green)',
            strokeWidth: '1px',
            borderWidth: '1px',
            opacity: 0.5,
            label: {
              style: {
                background: 'var(--secondary-3)'
              },
              text: 'Swap Fees',
              position: 'left',
              offsetX: 60,
              offsetY: 16
            }
          },
          {
            y: poolPrice,
            fillColor: 'var(--secondary-2)',
            strokeColor: 'var(--purple)',
            strokeWidth: '1px',
            label: {
              style: {
                background: 'var(--secondary-3)'
              },
              text: `${dictionary.actions.swap.poolPrice}: ${priceAnnotationPrefix}${currencyFormatter(
                poolPrice,
                false,
                3
              )}${priceAnnotationSuffix}`,
              offsetY: 6
            }
          },
          {
            y: oraclePrice,
            fillColor: 'var(--secondary-2)',
            strokeColor: 'var(--secondary-2)',
            strokeWidth: '2px',
            label: {
              style: {
                background: 'var(--secondary-3)'
              },
              text: `${dictionary.actions.swap.oraclePrice}: ${priceAnnotationPrefix}${currencyFormatter(
                oraclePrice,
                false,
                3
              )}${priceAnnotationSuffix}`,
              offsetX: -180,
              offsetY: 6
            }
          }
        ],
        points: []
      },
      series: [
        {
          name: dictionary.actions.swap.swapsGraph.title,
          data: chartData
        }
      ],
      xaxis: {
        title: {
          text: currentPool ? `${dictionary.actions.swap.sellQuantity} ${currentPool!.symbol}` : '—'
        },
        labels: {
          padding: 0,
          formatter: (value: number) => {
            return currencyFormatter(value, false, 3, false);
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
          text: currentPool && outputToken ? `${currentPool?.symbol}/${outputToken?.symbol} price` : '—'
        },
        labels: {
          padding: 20,
          formatter: (value: number) => {
            return currencyFormatter(value, false, 2);
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
            show: true,
            offsetX: 0
          }
        },
        yaxis: {
          lines: {
            show: true,
            offsetX: 0
          }
        }
      },
      stroke: {
        width: [1],
        dashArray: [0],
        // lineCap: "round",
        curve: 'straight'
      },
      plotOptions: {},
      colors: ['#f0f0f0'],
      legend: {
        show: true
      }
    });
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
      currentChart?.removeAnnotation('my-swap');
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
          id: 'my-swap',
          x: parseFloat(tokenInputString),
          y: swapPrice,
          marker: {
            size: 3,
            fillColor: '#c53727'
          },
          label: {
            borderColor: '#2badff',
            text: `${dictionary.actions.swap.mySwap}: ${tokenInputString} ${currentPool?.symbol ?? ''} → ${
              outputToken?.symbol ?? ''
            }`
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
      {loading && (
        <div className="overlay-message">
          <LoadingOutlined />
        </div>
      )}
      <div className="swaps-graph-container flex-centered"></div>
      <ReorderArrows component="swapsGraph" order={swapsRowOrder} setOrder={setSwapsRowOrder} />
    </div>
  );
}
