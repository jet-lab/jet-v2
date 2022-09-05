import { useEffect } from 'react';
import { atom, useRecoilState, useResetRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import axios from 'axios';
import { Cluster } from '../settings/settings';
import { CurrentMarket } from './market';
import { createDummyArray } from '../../utils/ui';

// Recent trades
const VOLUME_REFRESH_INTERVAL = 3600000;
const TRADES_REFRESH_INTERVAL = 5000;
const TRADES_DEPTH = 10;
export interface SerumTrade {
  market: string;
  size: number;
  price: number;
  orderId: string;
  time: number;
  side: string;
  feeCost: number;
  marketAddress: string;
}
export const RecentTrades = atom({
  key: 'recentTrades',
  default: createDummyArray(TRADES_DEPTH, 'orderId') as SerumTrade[]
});
export const OneDayVolume = atom({
  key: 'oneDayVolume',
  default: 0 as number
});
export const RecentTradesLoaded = atom({
  key: 'recentTradesLoaded',
  default: false as boolean
});

// Wrapper to provide contextual updates to Recent Trades
export function RecentTradesWrapper(props: { children: JSX.Element }) {
  const cluster = useRecoilValue(Cluster);
  const currentMarket = useRecoilValue(CurrentMarket);
  const [recentTrades, setRecentTrades] = useRecoilState(RecentTrades);
  const resetRecentTrades = useResetRecoilState(RecentTrades);
  const setOneDayVolume = useSetRecoilState(OneDayVolume);
  const setRecentTradesLoaded = useSetRecoilState(RecentTradesLoaded);

  // On init / new market pair
  useEffect(() => {
    // Trades
    function setTrades() {
      if (currentMarket && cluster !== 'devnet') {
        // TODO: Replace endpoint with our own
        axios
          .get(`https://event-history-api-candles.herokuapp.com/trades/address/${currentMarket.address.toString()}`)
          .then(({ data }) => {
            const trades = data.data;
            if (trades) {
              if (recentTrades[0]?.time !== trades[0]?.time) {
                setRecentTrades(trades);
              }
            }
          })
          .catch(err => err);
      }
      setRecentTradesLoaded(true);
    }

    // 24 Hour volume
    function setDayVolume() {
      if (currentMarket && cluster !== 'devnet') {
        // TODO: Replace endpoint with our own
        axios
          .get(`https://serum-api.bonfida.com/volumes/${currentMarket.address.toString()}`)
          .then(({ data }) => {
            const volume = data.data;
            if (volume) {
              setOneDayVolume(volume.volumeUsd);
            }
          })
          .catch(err => err);
      }
    }

    resetRecentTrades();
    setRecentTradesLoaded(false);

    const tradesInterval = setInterval(setTrades, TRADES_REFRESH_INTERVAL);
    setTrades();
    const volumeInterval = setInterval(setDayVolume, VOLUME_REFRESH_INTERVAL);
    setDayVolume();
    return () => {
      clearInterval(volumeInterval);
      clearInterval(tradesInterval);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cluster, currentMarket]);

  return <>{props.children}</>;
}
