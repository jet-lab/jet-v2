interface FixedTermSlice {
    markets: Record<string, {}>;
    selectedMarketKey: string;
    marketsLastUpdated: number;
    openOrders: Record<string, OpenOrders>;
    openPositions: Record<string, OpenPositions>;
    updateMarket: (update: {}) => void;
    initAllMarkets: (update: Record<string, {}>) => void;
    selectMarket: (address: string) => void;
    updateOpenOrders: (update: OpenOrders) => void;
    updateOpenPositions: (update: OpenPositions) => void;
}