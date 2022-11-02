import { BondMarketConfig } from '@jet-lab/margin';
import { formatDuration, intervalToDuration } from 'date-fns';

export const marketToString = (marketConfig: BondMarketConfig): string => {
  const duration = formatDuration(intervalToDuration({ start: 0, end: 1000 * marketConfig.borrowDuration }), {
    delimiter: '-',
    format: ['hours', 'days']
  });
  return `${duration} ${marketConfig.symbol}`;
};
