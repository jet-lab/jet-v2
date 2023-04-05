import { FixedTermMarketConfig, MarginAccount } from '@jet-lab/margin';
import { formatDuration, intervalToDuration } from 'date-fns';

export const friendlyMarketName = (symbol: string, tenor: number): string => {
  const duration = formatDuration(intervalToDuration({ start: 0, end: 1000 * tenor }), {
    delimiter: '-',
    format: ['minutes', 'hours', 'days']
  });
  return `${duration} ${symbol}`;
};

export const marketToString = (market: FixedTermMarketConfig): string => {
  return friendlyMarketName(market.symbol, market.borrowTenor);
};

export const feesCalc = (rate: number, interest: number) => {
  const feesRate = MarginAccount.ORIGINATION_FEE;
  const ratio = rate / feesRate;
  const fees = interest / ratio;
  return fees;
};
