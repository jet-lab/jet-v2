import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '../state/settings/localization/localization';
import { CurrentMarket, MarketPrice } from '../state/trade/market';
import { TradeViewOrder, TradeRowOrder } from '../state/views/views';
import { animateViewIn } from '../utils/ui';
import { useCurrencyFormatting } from '../utils/currency';
import { AccountSnapshot } from '../components/misc/AccountSnapshot';
import { OrderEntry } from '../components/TradeView/OrderEntry/OrderEntry';
import { Orderbook } from '../components/TradeView/Orderbook/Orderbook';
import { RecentTrades } from '../components/TradeView/RecentTrades';
import { PairSelector } from '../components/TradeView/PairSelector/PairSelector';
import { CandleStickChart } from '../components/TradeView/CandleStickChart';
import { PairRelatedAccount } from '../components/tables/PairRelatedAccount';

export function TradeView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const currentMarket = useRecoilValue(CurrentMarket);
  const marketPrice = useRecoilValue(MarketPrice);
  const { currencyFormatter } = useCurrencyFormatting();

  // Update page title with market info or just localize
  useEffect(() => {
    if (currentMarket && marketPrice) {
      document.title = `${currencyFormatter(marketPrice, false)} - ${currentMarket.baseSymbol ?? '—'} / ${
        currentMarket.quoteSymbol ?? '—'
      } | Jet Protocol`;
    } else {
      document.title = `${dictionary.tradeView.title} | Jet Protocol`;
    }
  }, [currentMarket, marketPrice, dictionary.tradeView.title, currencyFormatter]);

  // Row of Order Entry, Orderbook and Recent Trades components
  const rowOrder = useRecoilValue(TradeRowOrder);
  const rowComponents: Record<string, JSX.Element> = {
    orderEntry: <OrderEntry key="orderEntry" />,
    orderbook: <Orderbook key="orderbook" />,
    recentTrades: <RecentTrades key="recentTrades" />
  };
  const tradeRow = (): JSX.Element => {
    const tradeRowComponents: JSX.Element[] = [];
    for (const component of rowOrder) {
      tradeRowComponents.push(rowComponents[component]);
    }
    return (
      <div key="viewRow" className="view-row trade-row">
        {tradeRowComponents}
      </div>
    );
  };

  // Trade view with ordered components
  const viewOrder = useRecoilValue(TradeViewOrder);
  const viewComponents: Record<string, JSX.Element> = {
    accountSnapshot: <AccountSnapshot key="accountSnapshot" />,
    pairSelector: <PairSelector key="pairSelector" />,
    tradeRow: tradeRow(),
    candleStickChart: <CandleStickChart key="candleStickChart" />,
    pairRelatedAccount: <PairRelatedAccount key="pairRelatedAccount" />
  };
  const tradeView = (): JSX.Element => {
    const tradeViewComponents: JSX.Element[] = [];
    for (const component of viewOrder) {
      tradeViewComponents.push(viewComponents[component]);
    }
    return <div className="trade-view view">{tradeViewComponents}</div>;
  };

  useEffect(() => animateViewIn(), []);
  return tradeView();
}
