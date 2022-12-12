import { FixedTermMarketConfig } from '@jet-lab/margin';
import { formatDuration, intervalToDuration } from 'date-fns';

export const friendlyMarketName = (symbol: string, tenor: number): string => {
  const duration = formatDuration(intervalToDuration({ start: 0, end: 1000 * tenor }), {
    delimiter: '-',
    format: ['hours', 'days']
  });
  return `${duration} ${symbol}`;
};

export const marketToString = (market: FixedTermMarketConfig): string => {
  return friendlyMarketName(market.symbol, market.borrowTenor);
};
