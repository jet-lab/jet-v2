import { useEffect, useState } from 'react';
import { atom, useRecoilState, useSetRecoilState, useRecoilValue } from 'recoil';
import { PublicKey } from '@solana/web3.js';
import { Market as MarginMarket, Orderbook as MarginOrderbook } from '@jet-lab/margin';
import { localStorageEffect } from '../effects/localStorageEffect';
import { Cluster } from '../settings/settings';
import { Dictionary } from '../settings/localization/localization';
import { ActionRefresh, ACTION_REFRESH_INTERVAL } from '../actions/actions';
import { useProvider } from '../../utils/jet/provider';
import { useMarginConfig } from '../../utils/jet/marginConfig';
import { notify } from '../../utils/notify';
import { getAccountInfoAndSubscribe } from '../../utils/subscribe';

// Market
export const Markets = atom({
  key: 'markets',
  default: {} as Record<string, MarginMarket>,
  dangerouslyAllowMutability: true
});
export const CurrentMarket = atom({
  key: 'currentMarket',
  default: undefined as MarginMarket | undefined,
  dangerouslyAllowMutability: true
});
export const MarketPairs = atom({
  key: 'marketPairs',
  default: [] as string[]
});
export const CurrentMarketPair = atom({
  key: 'currentMarketPair',
  default: 'SOL/USDC' as string,
  effects: [localStorageEffect('jetAppCurrentMarketPair')]
});
export const MarketsInit = atom({
  key: 'marketsInit',
  default: false as boolean
});
export const ORDERBOOOK_DEPTH = 8;
export const Orderbook = atom({
  key: 'orderbook',
  default: undefined as MarginOrderbook | undefined,
  dangerouslyAllowMutability: true
});
export const MarketPrice = atom({
  key: 'marketPrice',
  default: undefined as number | undefined
});

// Wrapper to provide contextual updates to Market
export function MarketWrapper(props: { children: JSX.Element[] }) {
  const cluster = useRecoilValue(Cluster);
  const config = useMarginConfig();
  const { programs, provider } = useProvider();
  const dictionary = useRecoilValue(Dictionary);
  const [marketsInit, setmarketsInit] = useRecoilState(MarketsInit);
  const [markets, setMarkets] = useRecoilState(Markets);
  const [currentMarket, setCurrentMarket] = useRecoilState(CurrentMarket);
  const setMarketPairs = useSetRecoilState(MarketPairs);
  const currentMarketPair = useRecoilValue(CurrentMarketPair);
  const [bidsSubscription, setBidsSubscription] = useState<{ id: number; buffer: Buffer }>();
  const [asksSubscription, setAsksSubscription] = useState<{ id: number; buffer: Buffer }>();
  const [orderbook, setOrderbook] = useRecoilState(Orderbook);
  const setMarketPrice = useSetRecoilState(MarketPrice);
  const actionRefresh = useRecoilValue(ActionRefresh);

  // Setup market pairs on init
  useEffect(() => {
    if (!config) {
      return;
    }

    const marketPairs: string[] = [];
    for (const market of Object.values(config.markets)) {
      marketPairs.push(`${market.baseSymbol}/${market.quoteSymbol}`);
    }
    setMarketPairs(marketPairs);
  }, [cluster, config, setMarketPairs]);

  // On marketsInit and any cluser / marketPair change, get currentMarket and subscribe to orderbook and subscribe to orderbook
  useEffect(() => {
    async function currentMarketOrderbookSubscribe() {
      const currentMarket = Object.values(markets).filter(market => currentMarketPair === market.name)[0];
      setCurrentMarket(currentMarket);

      // Remove current subscriptions
      if (bidsSubscription?.id && asksSubscription?.id) {
        provider.connection.removeAccountChangeListener(bidsSubscription.id);
        provider.connection.removeAccountChangeListener(asksSubscription.id);
      }

      // Subscribe to bids and asks
      const bidsSub: any = {};
      bidsSub.id = await getAccountInfoAndSubscribe(
        provider.connection,
        new PublicKey(currentMarket.marketConfig.bids),
        accountInfo => {
          if (accountInfo) {
            bidsSub.buffer = accountInfo.data;
          }
        }
      );
      const asksSub: any = {};
      asksSub.id = await getAccountInfoAndSubscribe(
        provider.connection,
        new PublicKey(currentMarket.marketConfig.asks),
        accountInfo => {
          if (accountInfo) {
            asksSub.buffer = accountInfo.data;
          }
        }
      );
      setBidsSubscription(bidsSub);
      setAsksSubscription(asksSub);
    }
    currentMarket && currentMarketOrderbookSubscribe();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cluster, marketsInit, currentMarketPair]);

  // On bidsBuffer or asksBuffer update, load Orderbook
  useEffect(() => {
    async function getOrderbook() {
      if (!currentMarket || !bidsSubscription || !asksSubscription) {
        return;
      }

      const newOrderbook = MarginOrderbook.load({
        market: currentMarket.serum,
        bidsBuffer: bidsSubscription.buffer,
        asksBuffer: asksSubscription.buffer
      });
      const bids = newOrderbook.getBids();
      const oldBids = orderbook?.getBids();
      const asks = newOrderbook.getAsks();
      const oldAsks = orderbook?.getAsks();
      // If latest are the same, don't update UI
      if (JSON.stringify([bids, asks]) !== JSON.stringify([oldBids, oldAsks])) {
        if (bids.length && asks.length) {
          const bestBid = bids[0][0];
          const bestAsk = asks[0][0];
          const fills = await currentMarket.serum.loadFills(provider.connection);
          const last = fills[0]?.price;
          const marketPrice =
            bestBid && bestAsk
              ? last
                ? [bestBid, bestAsk, last].sort((a, b) => a - b)[1]
                : (bestBid + bestAsk) / 2
              : undefined;

          setMarketPrice(marketPrice);
        }
        setOrderbook(newOrderbook);
      }
    }

    getOrderbook();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bidsSubscription?.buffer, asksSubscription?.buffer]);

  // Fetch market on init / market change
  // Re-fetch on actionRefresh
  useEffect(() => {
    async function getMarkets() {
      if (!programs) {
        return;
      }

      try {
        const markets = await MarginMarket.loadAll(programs);
        setMarkets(markets);
        setmarketsInit(true);
      } catch (err) {
        console.error(err);
        notify(
          dictionary.tradeView.errorMessages.noMarket.replaceAll('{{MARKET_NAME}}', currentMarket?.name ?? ''),
          dictionary.tradeView.errorMessages.noMarketDescription,
          'error'
        );
      }
    }

    getMarkets();
    const marketsInterval = setInterval(getMarkets, ACTION_REFRESH_INTERVAL);
    return () => clearInterval(marketsInterval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cluster, programs, actionRefresh]);

  return <>{props.children}</>;
}
