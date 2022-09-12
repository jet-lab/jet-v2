import {
  formatPubkey,
  formatRate,
  formatLeverage,
  formatRiskIndicator,
  formatRemainder,
  formatMarketPair
} from '../format';

describe('Utils - Formatting', () => {
  describe('formatPubkey', () => {
    it('should return the first and last 4 digits of a pubkey, with ellipsis between', () => {
      const key = 'this-long-string-should-represent-a-public-key';
      const result = formatPubkey(key, 4);
      expect(result).toBe('this...-key');
    });
  });

  describe('formatRate', () => {
    it('should return a rate as a formatted string, with percentage sign to specified decimal place', () => {
      const rate = 0.9543409;
      const result = formatRate(rate, 3);
      expect(result).toBe('95.434%');
    });
  });

  describe('formatLeverage', () => {
    it('should return a given leverage value as a formatted string with an "x"', () => {
      const leverage = 124;
      const result = formatLeverage(leverage, 3);
      expect(result).toBe('1.24x');
    });
  });

  describe('formatRiskIndicator', () => {
    it('should return the formatted risk indicator', () => {
      const risk = 0.825;
      const result = formatRiskIndicator(risk);
      expect(result).toBe('0.82');
    });

    it('should remove trailing 0s', () => {
      const risk = 0.825;
      const result = formatRiskIndicator(risk, 3);
      expect(result).toBe('0.825');
    });

    it('should cap the indicator at 1', () => {
      const risk = 2;
      const result = formatRiskIndicator(risk);
      expect(result).toBe('>1');
    });

    it('should handle missing risks', () => {
      const result = formatRiskIndicator(undefined);
      expect(result).toBe('0');
    });
  });

  describe('formatRemainder', () => {
    it('should remove trailing 0s', () => {
      const result = formatRemainder('19.034380000');
      expect(result).toBe('19.03438');
    });
  });

  describe('formatMarketPair', () => {
    it('should format currency pairs', () => {
      const result = formatMarketPair('SOL/USDC');
      expect(result).toBe('SOL / USDC');
    });
  });
});
