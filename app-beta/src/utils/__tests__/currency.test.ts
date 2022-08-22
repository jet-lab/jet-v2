import { currencyFormatter, totalAbbrev } from '../currency';

describe('Utils - Currency', () => {
  const testAmount = 20000;
  const modifier = 0.000001;
  const price = 100;
  const digits = 8;

  describe('currencyFormatter', () => {
    it('should handle USD values', () => {
      const result = currencyFormatter(testAmount, true);
      expect(result).toEqual('$20,000.00');
    });

    it('should handle Crypto values', () => {
      const result = currencyFormatter(testAmount, false);
      expect(result).toEqual('20,000');
    });

    it('should accept a digit parameters', () => {
      const result = currencyFormatter(testAmount + modifier, false, digits);
      expect(result).toEqual('20,000.000001');
    });

    it('should accept discard the trailing 0s', () => {
      const result = currencyFormatter(testAmount, false, digits);
      expect(result).toEqual('20,000');
    });
  });

  describe('totalAbbrev', () => {
    it('should default to USD', () => {
      const result = totalAbbrev(testAmount);
      expect(result).toEqual('$20.0K');
    });

    it('should accept a `price` parameter', () => {
      const result = totalAbbrev(testAmount, price);
      expect(result).toEqual('$20.0K');
    });

    it('should accept a `native` parameter', () => {
      const result = totalAbbrev(testAmount, price, true);
      expect(result).toEqual('20.0K');
    });

    it('should accept a `digits` parameter', () => {
      const result = totalAbbrev(testAmount, price, false, digits);
      expect(result).toEqual('$2.0M');
    });

    it('should ignore the digits parameter is the native parameter is true', () => {
      const result = totalAbbrev(testAmount, price, true, digits);
      expect(result).toEqual('20.0K');
    });
  });
});
