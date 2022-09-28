export interface MockMarket {
  name: string;
  tenor: number;
  rateMidpoint: number;
  totalLent: number;
  totalBorrowed: number;
  change12hrs: number;
  change24hrs: number;
  volume: number;
  dailyRange: [number, number];
}

export interface MockOrder {
  base_size: number;
  quote_size: number;
  price: number;
}

export interface MockBook {
  market: string;
  asks: MockOrder[];
  bids: MockOrder[];
}

const USDC_DECIMALS = 6;

export const generateMarkets = (): MockMarket[] =>
  new Array(3).fill(0).map((_, index) => {
    const totalLent = Math.random() * 4000000;
    const totalBorrowed = Math.random() * 4000000;
    const volume = totalLent + totalBorrowed;
    return {
      name: `${index + 1}-day${index === 0 ? '' : 's'} USDC`,
      tenor: index + 1 * 24 * 60 * 60,
      rateMidpoint: (Math.random() + index) / 100,
      totalLent,
      totalBorrowed,
      change12hrs: Math.random() * 2.5,
      change24hrs: Math.random() * 2.5,
      volume,
      dailyRange: [
        volume / 2 - (volume / 2) * (Math.random() * 0.1),
        volume / 2 + (volume / 2) * (Math.random() * 0.1)
      ],
      decimals: USDC_DECIMALS
    };
  });

export const generateOrders = (market: MockMarket, deviation: number, type: string): MockOrder[] =>
  new Array(50).fill(0).map(() => {
    const marketRate = market.rateMidpoint;
    const base_rate = type === 'asks' ? marketRate + deviation : marketRate - deviation;
    const quote_size = Math.ceil(Math.random() * 5000);
    const base_size = quote_size + base_rate * quote_size;
    const price = base_size / quote_size;
    return {
      quote_size,
      base_size,
      price
    };
  });

export const generateOrderBook = (market: MockMarket): MockBook => {
  const deviation = Math.random() * 0.05;
  const baseAsks = market.rateMidpoint + deviation;
  const baseBids = market.rateMidpoint - deviation;
  return {
    market: market.name,
    asks: generateOrders(market, baseAsks, 'asks').sort((a, b) => b.price - a.price),
    bids: generateOrders(market, baseBids, 'bids').sort((a, b) => a.price - b.price)
  };
};
