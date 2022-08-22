import { useEffect, useState, useMemo } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { CurrentPoolSymbol } from '../../state/borrow/pools';
import { PriceHistoryLoading, PriceHistory } from '../../state/trade/priceHistory';
import { Sparklines, SparklinesLine } from 'react-sparklines';
import { Typography, Skeleton } from 'antd';

export function PriceHistorySparkline(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const currentPoolSymbol = useRecoilValue(CurrentPoolSymbol);
  const priceHistory = useRecoilValue(PriceHistory);
  const oneHourPriceHistory = priceHistory['1'];
  const twelveHourPriceHistory = priceHistory['5'];
  const oneDayPriceHistory = priceHistory['15'];
  const oneWeekPriceHistory = priceHistory['60'];
  const priceHistoryLoading = useRecoilValue(PriceHistoryLoading);
  const [selectedPriceHistory, setSelectedPriceHistory] = useState(oneWeekPriceHistory);
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
  const [currentTimeframe, setCurrentTimeframe] = useState(timeframeOptions[timeframeOptions.length - 1].label);
  const { Text } = Typography;
  useEffect(() => {
    for (const timeframe of timeframeOptions) {
      if (timeframe.label === currentTimeframe) {
        setSelectedPriceHistory(timeframe.priceHistory);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [timeframeOptions, currentTimeframe, priceHistoryLoading]);

  return (
    <div className="sparkline-chart view-element-item view-element-item-hidden">
      <div
        className={`flex-centered column ${
          selectedPriceHistory && currentPoolSymbol !== 'USDC'
            ? selectedPriceHistory.percentageChange > 0
              ? 'sparkline-chart-up'
              : 'sparkline-chart-down'
            : ''
        }`}>
        {selectedPriceHistory && currentPoolSymbol !== 'USDC' ? (
          <Sparklines data={selectedPriceHistory.prices} width={300} height={40}>
            <SparklinesLine />
          </Sparklines>
        ) : (
          <Skeleton className="align-center" paragraph={false} active={!selectedPriceHistory && priceHistoryLoading} />
        )}
      </div>
      {selectedPriceHistory && currentPoolSymbol !== 'USDC' && (
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
      )}
    </div>
  );
}
