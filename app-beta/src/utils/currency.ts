// Format USD or crypto with default or desired decimals
export function currencyFormatter(value: number, usd: boolean, digits?: number): string {
  const amount = Math.floor(value * 10 ** (digits ?? 2)) / 10 ** (digits ?? 2);
  let currencyFormat: Intl.NumberFormat;
  let uiCurrency: string;
  if (usd) {
    currencyFormat = new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: digits ?? 2
    });
  } else {
    currencyFormat = new Intl.NumberFormat('en-US', {
      minimumFractionDigits: digits ?? 8,
      maximumFractionDigits: digits ?? 8
    });
  }

  // Set and strip trailing 0's / unnecessary decimal if not USD
  uiCurrency = currencyFormat.format(amount);
  if (!usd) {
    while (
      uiCurrency.indexOf('.') !== -1 &&
      (uiCurrency[uiCurrency.length - 1] === '0' || uiCurrency[uiCurrency.length - 1] === '.')
    ) {
      uiCurrency = uiCurrency.substring(0, uiCurrency.length - 1);
    }
  }

  return uiCurrency;
}

// Abbreviate large totals
export function totalAbbrev(total: number, price?: number, native?: boolean, digits?: number): string {
  let t = total;
  if (price && native === false) {
    t = total * price;
  }

  if (t > 1000000000) {
    return `${native ? '' : '$'}${(t / 1000000000).toFixed(1)}B`;
  } else if (t > 1000000) {
    return `${native ? '' : '$'}${(t / 1000000).toFixed(1)}M`;
  } else if (t > 1000) {
    return `${native ? '' : '$'}${(t / 1000).toFixed(1)}K`;
  } else {
    return currencyFormatter(t, !native, native ? digits : 2);
  }
}
