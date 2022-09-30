import { assert } from 'chai';
import {
  base_to_quote,
  calculate_implied_price,
  price_to_rate,
  quote_to_base,
  rate_to_price
} from '@jet-lab/jet-bonds-client';
import { bigIntToBn, bnToBigInt } from '@jet-lab/margin';
import { BN } from '@project-serum/anchor';

describe('wasm-utils', () => {
  it('rate price conversions', () => {
    // Start with BN, convert to BigInt then to price values
    const base = new BN(1_200_000);
    const quote = new BN(1_000_000);
    const tenor = new BN(13_000);

    const price = calculate_implied_price(bnToBigInt(base), bnToBigInt(quote));
    const rate = price_to_rate(price, bnToBigInt(tenor));

    const derivedPrice = rate_to_price(rate, bnToBigInt(tenor));
    assert(
      price.toString() === derivedPrice.toString(),
      'failed to convert between rate and price' +
        '\nprice: [' +
        price +
        ']' +
        '\nderived price: [' +
        derivedPrice +
        ']'
    );
  });
  it('BN and BigInt conversions', () => {
    const bnOneThousand = new BN(1_000);
    const bigIntOneThousand = BigInt(1_000);

    const convertedBItoBN = bigIntToBn(bigIntOneThousand);
    const convertedBNtoBI = bnToBigInt(bnOneThousand);

    assert(
      bnOneThousand.toString() === convertedBItoBN.toString(),
      'failed to convert bigint to BN' + '\nBN: [' + bnOneThousand + ']\nconverted bigint: [' + convertedBItoBN + ']'
    );
    assert(
      bigIntOneThousand.toString() === convertedBNtoBI.toString(),
      'failed to convert BN to bigint' +
        '\nbigint: [' +
        bigIntOneThousand +
        ']\nconverted BN: [' +
        convertedBNtoBI +
        ']'
    );
  });
  it('base quote and price conversions', () => {
    const base = new BN(1_200_000);
    const quote = new BN(1_000_000);
    const price = calculate_implied_price(bnToBigInt(base), bnToBigInt(quote));

    const derivedBase = bigIntToBn(quote_to_base(bnToBigInt(quote), price));
    const derivedQuote = bigIntToBn(base_to_quote(bnToBigInt(base), price));
    assert(
      base.toString() === derivedBase.toString(),
      'failed to derive base from quote and price\nBase: [' +
        base +
        ']\nDerived base: [' +
        derivedBase +
        ']\nPrice: [' +
        price +
        ']'
    );
    // NOTE: accounts for rounding
    assert(
      quote.toString() === derivedQuote.add(new BN(1)).toString(),
      'failed to derive quote from base and price' +
        '\nQuote: [' +
        quote +
        ']\nDerived quote: [' +
        derivedQuote +
        ']\nPrice: [' +
        price +
        ']'
    );
  });
});
