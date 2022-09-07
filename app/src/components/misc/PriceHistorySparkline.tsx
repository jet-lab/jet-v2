import { useEffect, useState, useMemo } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { CurrentPoolSymbol } from '../../state/borrow/pools';
import { PriceHistoryLoading, PriceHistory } from '../../state/trade/priceHistory';
import { Sparklines, SparklinesLine } from 'react-sparklines';
import { Typography, Skeleton } from 'antd';

// Simple sparkline for an asset's price history
export function PriceHistorySparkline(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const currentPoolSymbol = useRecoilValue(CurrentPoolSymbol);
  const isStablecoin = currentPoolSymbol === 'USDC' || currentPoolSymbol === 'USDT';
  const priceHistory = useRecoilValue(PriceHistory);
  const oneHourPriceHistory = priceHistory['1'];
  const twelveHourPriceHistory = priceHistory['5'];
  const oneDayPriceHistory = priceHistory['15'];
  const oneWeekPriceHistory = priceHistory['60'];
  const priceHistoryLoading = useRecoilValue(PriceHistoryLoading);
  const timeframeOptions = useMemo(
    () => [
      {
        label: '1H',
        priceHistory: oneHourPriceHistory
      },
      {
        label: '12H',
        priceHistory: twelveHourPriceHistory
      },
      {
        label: '1D',
        priceHistory: oneDayPriceHistory
      },
      {
        label: '1W',
        priceHistory: oneWeekPriceHistory
      }
    ],
    [oneHourPriceHistory, twelveHourPriceHistory, oneDayPriceHistory, oneWeekPriceHistory]
  );
  const [selectedPriceHistory, setSelectedPriceHistory] = useState(oneWeekPriceHistory);
  const [currentTimeframe, setCurrentTimeframe] = useState(timeframeOptions[timeframeOptions.length - 1].label);
  const { Text } = Typography;

  // Update the price history when the timeframe is updated
  useEffect(() => {
    const selectedHistory = timeframeOptions.filter(tf => tf.label === currentTimeframe)[0];
    setSelectedPriceHistory(selectedHistory.priceHistory);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [timeframeOptions, currentTimeframe, priceHistoryLoading]);

  // Returns the styling classname based on if price has increased/decreased
  function getSparklineClass() {
    // Default to no styling (for stablecoins)
    let className = '';
    if (selectedPriceHistory && !isStablecoin) {
      // For non-stablecoins, check if change is positive
      if (selectedPriceHistory.percentageChange > 0) {
        className = 'sparkline-chart-up';
      } else {
        className = 'sparkline-chart-down';
      }
    }

    return className;
  }

  // Renders the sparkline
  function renderSparkline() {
    // Default to a straight skeleton placeholder, show as loading if applicable
    let render = (
      <Skeleton className="align-center" paragraph={false} active={!selectedPriceHistory && priceHistoryLoading} />
    );
    // If not viewing a stablecoin
    if (selectedPriceHistory && !isStablecoin) {
      render = (
        <Sparklines data={selectedPriceHistory.prices} width={300} height={40}>
          <SparklinesLine />
        </Sparklines>
      );
    }

    return render;
  }

  // Render the options for the sparkline
  function renderOptions() {
    // Only render if loaded and not viewing a stablecoin
    let render = <></>;
    if (!isStablecoin) {
      render = (
        <div className="sparkline-chart-options flex align-center justify-between">
          <Text italic>{dictionary.common.priceHistory}</Text>
          <div className="sparkline-chart-options-items flex-centered">
            {timeframeOptions.map(option => (
              <Text
                key={option.label}
                className={option.label === currentTimeframe ? 'active' : ''}
                onClick={() => setCurrentTimeframe(option.label)}>
                {option.label}
              </Text>
            ))}
          </div>
        </div>
      );
    }

    return render;
  }

  return (
    <div className="sparkline-chart view-element-item view-element-item-hidden">
      <div className={`flex-centered column ${getSparklineClass()}`}>{renderSparkline()}</div>
      {renderOptions()}
    </div>
  );
}
