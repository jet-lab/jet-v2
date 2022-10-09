import { useCallback } from 'react';
import { useRecoilValue } from 'recoil';
import { BN } from '@project-serum/anchor';
import { TokenAmount } from '@jet-lab/margin';
import { FiatCurrency, USDConversionRates } from '../state/settings/settings';

// Default decimal precision for tokens
export const DEFAULT_DECIMALS = 4;

// Hook for currency formatting functions
export function useCurrencyFormatting() {
  const fiatCurrency = useRecoilValue(FiatCurrency);
  const conversionRates = useRecoilValue(USDConversionRates);

  // Format USD or crypto with default or desired decimals
  const currencyFormatter = useCallback(
    (value: number, fiatValues?: boolean, decimals?: number, ceil?: boolean, accounting?: boolean): string => {
      const roundedValue = Math.round(value * 10 ** (decimals ?? 2)) / 10 ** (decimals ?? 2);
      const convertedValue =
        fiatCurrency !== 'USD' ? roundedValue * conversionRates[fiatCurrency] : roundedValue;

      const currencyFormat = new Intl.NumberFormat(navigator.language, {
        style: fiatValues ? 'currency' : undefined,
        currency: fiatValues ? fiatCurrency : undefined,
        currencyDisplay: fiatValues ? 'narrowSymbol' : undefined,
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals ?? 2,
        currencySign: accounting && fiatValues ? 'accounting' : undefined
      });

      let uiCurrency = currencyFormat.format(convertedValue);
      if (!fiatValues && !accounting) {
        // Set and strip trailing 0's / unnecessary decimals
        while (
          uiCurrency.indexOf('.') !== -1 &&
          (uiCurrency[uiCurrency.length - 1] === '0' || uiCurrency[uiCurrency.length - 1] === '.')
        ) {
          uiCurrency = uiCurrency.substring(0, uiCurrency.length - 1);
        }
      }

      return uiCurrency;
    },
    [conversionRates, fiatCurrency]
  );

  function dynamicDecimals(value: number) {
    if (value < 10) {
      return currencyFormatter(value, false, 8, undefined, true);
    } else if (value < 100) {
      return currencyFormatter(value, false, 7, undefined, true);
    } else if (value < 1000) {
      return currencyFormatter(value, false, 6, undefined, true);
    } else if (value < 10000) {
      return currencyFormatter(value, false, 5, undefined, true);
    } else if (value < 100000) {
      return currencyFormatter(value, false, 4, undefined, true);
    } else if (value < 1000000) {
      return currencyFormatter(value, false, 3, undefined, true);
    } else if (value < 10000000) {
      return currencyFormatter(value, false, 2, undefined, true);
    } else if (value < 100000000) {
      return currencyFormatter(value, false, 1, undefined, true);
    } else if (value < 1000000000) {
      return currencyFormatter(value, false, 0, undefined, true);
    } else if (value >= 1000000000) {
      return currencyFormatter(value / 1000000000, false, 1) + 'B';
    }
  }

  // Abbreviate large currency amounts
  function currencyAbbrev(
    total: number,
    fiatValues?: boolean,
    price?: number,
    decimals?: number,
    precision?: boolean,
    accounting?: boolean
  ): string {
    let t = total;
    if (price && fiatValues) {
      t = total * price;
    }

    // In all cases, truncate trillions and billions
    if (t > 1000000000000) {
      return currencyFormatter(t / 1000000000000, fiatValues, 1) + 'T';
    } else if (t > 1000000000) {
      return currencyFormatter(t / 1000000000, fiatValues, 1) + 'B';
    }

    if (precision) {
      if (fiatValues) {
        if (accounting) {
          return currencyFormatter(t, fiatValues, 2, undefined, true);
        }
        // Do not truncate fiat values under million
        return currencyFormatter(t, fiatValues, 2);
      } else {
        // If not fiat values, show up to the 9th character
        // with dynamic decimal places
        return dynamicDecimals(t);
      }
    }

    if (t > 1000000) {
      return currencyFormatter(t / 1000000, fiatValues, 1) + 'M';
    } else if (t > 1000) {
      return currencyFormatter(t / 1000, fiatValues, 1) + 'K';
    } else {
      return currencyFormatter(t, fiatValues, fiatValues ? 2 : decimals);
    }
  }

  return {
    currencyFormatter,
    currencyAbbrev
  };
}

// Get count of decimals for a number
export function getDecimalCount(number: number): number {
  if (!isNaN(number) && Math.floor(number) !== number && number.toString().includes('.')) {
    return number.toString().split('.')[1].length || 0;
  }
  if (!isNaN(number) && Math.floor(number) !== number && number.toString().includes('e')) {
    return parseInt(number.toString().split('e-')[1] || '0');
  }
  return 0;
}

// Create a TokenAmount from a number and decimal precision
export function getTokenAmountFromNumber(number: number, decimals: number): TokenAmount {
  let safeNum = Math.max(number * 10 ** decimals, 0);
  safeNum = Math.min(safeNum, Number.MAX_SAFE_INTEGER);
  safeNum = typeof safeNum === 'number' ? safeNum : 0;
  return new TokenAmount(new BN(safeNum), decimals);
}
