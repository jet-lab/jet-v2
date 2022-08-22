import { useEffect } from 'react';
import { atom, useRecoilValue, useResetRecoilState, useSetRecoilState } from 'recoil';
import axios from 'axios';
import { localStorageEffect } from '../effects/localStorageEffect';
import { CurrentMarket } from './market';
import {
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MONTH,
  SECONDS_PER_WEEK,
  SECONDS_PER_YEAR
} from '../../utils/time';

// Price history
export interface PeriodHistory {
  candles: number[][];
  prices: number[];
  pastPrice: number;
  percentageChange: number;
}
export type Period =
  | '1' /* Minute */
  | '3' /* 3 Minute */
  | '5' /* 5 Minute */
  | '15' /* 15 Minute */
  | '30' /* 30 Minute */
  | '60' /* Hour */
  | '120' /* 2 Hour */
  | '180' /* 3 Hour */
  | '240' /* 4 Hour */
  | '360' /* 6 Hour */
  | '1D' /* Day */;
export const periodOptions: Record<Period, Record<string, string | number>> = {
  '1': {
    // 2 hour depth
    timeframeIndexer: 'minute',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_HOUR * 2)
  },
  '3': {
    // 6 hour depth
    timeframeIndexer: 'threeMinute',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_HOUR * 6)
  },
  '5': {
    // 12 hour depth
    timeframeIndexer: 'fiveMinute',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_DAY / 2)
  },
  '15': {
    // 1 day depth
    timeframeIndexer: 'fifteenMinute',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_DAY)
  },
  '30': {
    // 3 day depth
    timeframeIndexer: 'thirtyMinute',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_DAY * 3)
  },
  '60': {
    // 1 week depth
    timeframeIndexer: 'hour',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_WEEK)
  },
  '120': {
    // 2 week depth
    timeframeIndexer: 'twoHour',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_WEEK * 2)
  },
  '180': {
    // 1 month depth
    timeframeIndexer: 'threeHour',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_MONTH)
  },
  '240': {
    // 1 month depth
    timeframeIndexer: 'fourHour',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_MONTH)
  },
  '360': {
    // 1.5 month depth
    timeframeIndexer: 'sixHour',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_MONTH * 1.5)
  },
  '1D': {
    // 6 month depth
    timeframeIndexer: 'day',
    timestampDepth: Math.round(Date.now() / 1000 - SECONDS_PER_YEAR / 2)
  }
};
export const CurrentPeriod = atom({
  key: 'currentPeriod',
  default: '30' as Period,
  effects: [localStorageEffect('jetAppCurrentPeriod')]
});
export const PriceHistory = atom({
  key: 'priceHistory',
  default: {} as Record<Period, PeriodHistory>,
  dangerouslyAllowMutability: true
});
export const PriceHistoryLoading = atom({
  key: 'priceHistoryLoading',
  default: true as boolean
});

// Wrapper to provide contextual updates to price history
export function PriceHistoryWrapper(props: { children: JSX.Element }) {
  const currentMarket = useRecoilValue(CurrentMarket);
  const setPriceHistory = useSetRecoilState(PriceHistory);
  const resetPriceHistory = useResetRecoilState(PriceHistory);
  const setPriceHistoryLoading = useSetRecoilState(PriceHistoryLoading);

  // Fetch price history on init / currentMarket change
  useEffect(() => {
    async function getPriceHistories() {
      if (!currentMarket) {
        return;
      }

      resetPriceHistory();
      setPriceHistoryLoading(true);
      const nowSeconds = Date.now() / 1000;
      const priceHistory: Record<string, PeriodHistory> = {};

      for (const period of Object.keys(periodOptions)) {
        const candles: number[][] = [];
        // TODO: Replace endpoint with our own
        axios
          .get(
            `https://event-history-api-candles.herokuapp.com/tv/history?symbol=${
              currentMarket.name
            }&resolution=${period}&from=${periodOptions[period as Period].timestampDepth}&to=${Math.round(nowSeconds)}`
          )
          .then(resp => {
            const apiCandles = resp.data;
            if (apiCandles.s === 'ok') {
              for (let i = 0; i < apiCandles.t.length; i++) {
                candles.push([
                  apiCandles.t[i],
                  parseFloat(apiCandles.o[i]), // Close time
                  parseFloat(apiCandles.h[i]), // Open price
                  parseFloat(apiCandles.l[i]), // High price
                  parseFloat(apiCandles.c[i]), // Low price
                  parseFloat(apiCandles.v[i]) // Close price
                ]);
              }
            }

            const prices: number[] = [];
            for (const ohlc of candles) {
              prices.push((ohlc[1] + ohlc[2]) / 2);
            }
            const currentPrice = prices[prices.length - 1];
            const pastPrice = prices[0];

            priceHistory[period] = {
              candles,
              prices,
              pastPrice,
              percentageChange: (currentPrice - pastPrice) / pastPrice
            };
            setPriceHistory(priceHistory);
          })
          .catch(err => err);
      }
      setPriceHistoryLoading(false);
    }

    getPriceHistories();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentMarket]);

  return <>{props.children}</>;
}

// Return array of candles for a token and period
export function useCandles(period: Period): number[][] | undefined {
  const priceHistory = useRecoilValue(PriceHistory);
  const periodHistory = priceHistory[period];
  if (periodHistory) {
    const { candles } = periodHistory;
    return candles?.length ? candles : undefined;
  }
}
