import { PublicKey } from '@solana/web3.js';

// Format and shorten a pubkey with ellipsis
export function formatPubkey(publicKey: PublicKey | string, halfLength = 4): string {
  const pubKey = publicKey.toString();
  return `${pubKey.substring(0, halfLength)}...${pubKey.substring(pubKey.length - halfLength)}`;
}

// Format rates
export function formatRate(rate: number, decimals?: number) {
  if (decimals) {
    return (rate * 100).toFixed(decimals).toLocaleString() + '%';
  }
  return parseFloat(formatRemainder((rate * 100).toFixed(2))).toLocaleString() + '%';
}

// Format leverage
export function formatLeverage(leverage: number, decimals?: number) {
  return parseFloat(formatRemainder((leverage / 100).toFixed(decimals ?? 1))).toLocaleString() + 'x';
}

// Format Risk Indicator
export function formatRiskIndicator(riskIndicator?: number, decimals?: number) {
  if (!riskIndicator) {
    return '0';
  } else if (riskIndicator > 1) {
    return '>1';
  } else {
    return formatRemainder(riskIndicator.toFixed(decimals ?? 2));
  }
}

// Format price impact on swaps or trades
export function formatPriceImpact(impact?: number) {
  if (!impact) {
    return '0';
  } else if (impact < 0.005) {
    return '<0.5%';
  } else {
    return `${(impact * 100).toFixed(2)}%`;
  }
}

// Remove trailing 0's and decimal if necessary
export function formatRemainder(value: string): string {
  return parseFloat(value).toString();
}

// Add space between / of market pairs
export function formatMarketPair(pair: string): string {
  return pair.split('/')[0] + ' / ' + pair.split('/')[1];
}

// Remove locale formatting from number string
export function fromLocaleString(num: string): string {
  const { format } = new Intl.NumberFormat(navigator.language);

  const decimalSign = format(0.1).substring(1, 2);
  const thousands = format(1000);
  const thousandSeparator = thousands.length === 5 ? thousands.substring(1, 2) : null;
  let strippedNum = num;
  // Remove thousands separator
  if (thousandSeparator) {
    strippedNum = strippedNum.replace(thousandSeparator, '');
  }
  // Replace , with . if applicable
  if (decimalSign !== '.') {
    strippedNum = strippedNum.replace(decimalSign, '.');
  }

  return strippedNum;
}


export const formatWithCommas = <T>(value: T) => `${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')