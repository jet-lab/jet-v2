import { useEffect, useRef, useState } from 'react';
import { useRecoilState, useSetRecoilState, useRecoilValue } from 'recoil';
import { Orderbook as MarginOrderbook } from '@jet-lab/margin';
import { Dictionary } from '../../../state/settings/localization/localization';
import { TradeRowOrder } from '../../../state/views/views';
import {
  Orderbook as OrderbookState,
  ORDERBOOOK_DEPTH,
  CurrentMarket,
  MarketPrice as MarketPriceState,
  CurrentMarketPair
} from '../../../state/trade/market';
import { OrderType, OrderPriceString } from '../../../state/trade/order';
import { animateDataUpdate } from '../../../utils/ui';
import { getDecimalCount, useCurrencyFormatting } from '../../../utils/currency';
import { Skeleton, Typography } from 'antd';
import { MarketPrice } from './MarketPrice';
import { ReorderArrows } from '../../misc/ReorderArrows';
import { MinusSquareOutlined, PlusSquareOutlined } from '@ant-design/icons';
import { Info } from '../../misc/Info';

interface SideOrder {
  cumulativeSize: number;
  price: number;
  size: number;
  sizePercent: number;
}
export function Orderbook(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter } = useCurrencyFormatting();
  const [tradeRowOrder, setTradeRowOrder] = useRecoilState(TradeRowOrder);
  const currentMarket = useRecoilValue(CurrentMarket);
  const currentMarketPair = useRecoilValue(CurrentMarketPair);
  const prevMarketPair = useRef(currentMarketPair);
  const orderbook = useRecoilValue(OrderbookState);
  const marketPrice = useRecoilValue(MarketPriceState);
  const setOrderType = useSetRecoilState(OrderType);
  const setOrderPriceString = useSetRecoilState(OrderPriceString);
  const prevOrderbook = useRef<MarginOrderbook | null>(null);
  const [orderbookData, setOrderbookData] = useState<{ bids: SideOrder[]; asks: SideOrder[] }>({ bids: [], asks: [] });
  const [spread, setSpread] = useState(0);
  const groupingOptions = [0.001, 0.01, 0.05, 0.1, 0.5, 1];
  const [grouping, setGrouping] = useState(groupingOptions[0]);
  const [loaded, setLoaded] = useState(false);
  const { Paragraph, Text } = Typography;

  // Get oderbook data for each side
  function getSideOrder(orders: number[][], totalSize: number) {
    const sideData: SideOrder[] = [];
    for (let i = 0; i < orders.length; i++) {
      const price = orders[i][0];
      const size = orders[i][1];
      const cumulativeSize = (sideData[i - 1]?.cumulativeSize || 0) + size;
      sideData.push({
        price,
        size,
        cumulativeSize,
        sizePercent: Math.round((cumulativeSize / (totalSize || 1)) * 100)
      });
    }

    return sideData;
  }

  // Group orders for a side by current grouping
  function groupSideOrders(sideOrders: number[][], isBids: boolean) {
    const groupFloors: Record<number, number> = {};
    for (let i = 0; i < sideOrders.length; i++) {
      if (typeof sideOrders[i] == 'undefined') {
        break;
      }
      const floor = isBids
        ? Math.floor(sideOrders[i][0] / grouping) * grouping
        : Math.ceil(sideOrders[i][0] / grouping) * grouping;
      if (typeof groupFloors[floor] == 'undefined') {
        groupFloors[floor] = sideOrders[i][1];
      } else {
        groupFloors[floor] = sideOrders[i][1] + groupFloors[floor];
      }
    }

    const sortedGroups = Object.entries(groupFloors)
      .map(entry => {
        return [+parseFloat(entry[0]).toFixed(getDecimalCount(grouping)), +entry[1]];
      })
      .sort((a: number[], b: number[]) => {
        if (!a || !b) {
          return -1;
        }
        return isBids ? b[0] - a[0] : a[0] - b[0];
      });
    return sortedGroups;
  }

  // Toggle through grouping options
  function toggleGrouping(higher: boolean) {
    if (higher && groupingOptions.indexOf(grouping) < groupingOptions.length - 1) {
      setGrouping(groupingOptions[groupingOptions.indexOf(grouping) + 1]);
    } else if (!higher && groupingOptions.indexOf(grouping) > 0) {
      setGrouping(groupingOptions[groupingOptions.indexOf(grouping) - 1]);
    }
  }

  // Pre-fill order entry on a row/side click
  function preFillOrder(price: number) {
    setOrderType('limit');
    setOrderPriceString(price.toString());
  }

  // Array of orderbook rows
  const orderbookRows = (): JSX.Element[] => {
    const rows: JSX.Element[] = [];
    for (let i = 0; i < ORDERBOOOK_DEPTH; i++) {
      const bid = orderbookData.bids[i];
      const ask = orderbookData.asks[i];
      rows.push(
        <div key={i} className="orderbook-body-row flex">
          <div
            className={`orderbook-body-row-half flex justify-between ${!bid ? 'no-interaction' : ''}`}
            onClick={() => (bid ? preFillOrder(bid.price) : null)}>
            <Paragraph type="secondary">
              {loaded ? (
                bid ? (
                  currencyFormatter(bid.size, false, currentMarket?.baseDecimals)
                ) : (
                  ''
                )
              ) : (
                <Skeleton paragraph={false} active />
              )}
            </Paragraph>
            <Paragraph type="success">
              {loaded ? (
                bid ? (
                  currencyFormatter(bid.price)
                ) : (
                  ''
                )
              ) : (
                <Skeleton className="align-right" paragraph={false} active />
              )}
            </Paragraph>
            <span className="orderbook-body-row-bg bid" style={{ width: `${bid ? bid.sizePercent : 0}%` }}></span>
          </div>
          <div
            className={`orderbook-body-row-half flex justify-between ${!ask ? 'no-interaction' : ''}`}
            onClick={() => (ask ? preFillOrder(ask.price) : null)}>
            <Paragraph type="danger">
              {loaded ? ask ? currencyFormatter(ask.price) : '' : <Skeleton paragraph={false} active />}
            </Paragraph>
            <Paragraph type="secondary">
              {loaded ? (
                ask ? (
                  currencyFormatter(ask.size, false, currentMarket?.baseDecimals)
                ) : (
                  ''
                )
              ) : (
                <Skeleton className="align-right" paragraph={false} active />
              )}
            </Paragraph>
            <span className="orderbook-body-row-bg ask" style={{ width: `${ask ? ask.sizePercent : 0}%` }}></span>
          </div>
        </div>
      );
    }
    return rows;
  };

  // On every orderbook update
  useEffect(() => {
    if (orderbook) {
      if (prevMarketPair.current !== currentMarketPair) {
        setLoaded(false);
        prevMarketPair.current = currentMarketPair;
      }

      // Cumulative order size and grouping for each side
      const sum = (total: number, [, size]: number[]) => total + size;
      const bids = orderbook.getBids();
      const asks = orderbook.getAsks();
      const totalSize = bids.reduce(sum, 0) + asks.reduce(sum, 0);
      const bidsSideOrders = getSideOrder(groupSideOrders(bids, true), totalSize);
      const asksSideOrders = getSideOrder(groupSideOrders(asks, false), totalSize);

      // Track previous data and update current
      prevOrderbook.current = orderbook;
      setOrderbookData({ bids: bidsSideOrders, asks: asksSideOrders });
      if (bidsSideOrders.length && asks.length) {
        setSpread(asksSideOrders[0].price - bidsSideOrders[0].price);
      }

      setLoaded(true);
      animateDataUpdate('flash-opacity-subtle', '.orderbook-body-row .ant-typography');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [orderbook, grouping]);

  return (
    <div className="orderbook view-element view-element-hidden flex column">
      <div className="orderbook-head view-element-item view-element-item-hidden">
        <ReorderArrows component="orderbook" order={tradeRowOrder} setOrder={setTradeRowOrder} />
        <div className="orderbook-head-top flex-centered">
          <Paragraph className="orderbook-head-top-title">{dictionary.tradeView.orderbook.title}</Paragraph>
        </div>
        <div className="orderbook-head-bottom flex-centered">
          <MarketPrice />
          <div className="orderbook-head-grouping flex-centered">
            <Paragraph type="secondary">{grouping}</Paragraph>
            <div className="orderbook-head-grouping-btns flex-centered">
              <PlusSquareOutlined onClick={() => toggleGrouping(true)} />
              <MinusSquareOutlined onClick={() => toggleGrouping(false)} />
            </div>
          </div>
        </div>
      </div>
      <div className="orderbook-body view-element-item view-element-item-hidden flex column">
        <div className="orderbook-body-top flex">
          <div className="orderbook-body-top-section flex justify-start">
            <Info term="bid">
              <Text className="small-accent-text info-element">{dictionary.tradeView.orderbook.bidSize}</Text>
            </Info>
          </div>
          <div className="orderbook-body-top-section flex justify-center">
            <Text className="small-accent-text">
              {dictionary.common.price} ({currentMarket?.quoteSymbol ?? '—'})
            </Text>
          </div>
          <div className="orderbook-body-top-section flex justify-end">
            <Info term="ask">
              <Text className="small-accent-text info-element">{dictionary.tradeView.orderbook.askSize}</Text>
            </Info>
          </div>
        </div>
        {orderbookRows()}
      </div>
      <div className="orderbook-footer view-element-item view-element-item-hidden flex-centered">
        <div className="orderbook-footer-spread flex align-center justify-between">
          <Paragraph type="secondary">{dictionary.tradeView.orderbook.spread}</Paragraph>
          {loaded ? (
            <Paragraph type="secondary">{loaded && marketPrice ? spread.toFixed(2) : '—'}</Paragraph>
          ) : (
            <Skeleton className="align-center" paragraph={false} active />
          )}
          <Paragraph type="secondary">
            {loaded && marketPrice ? ((spread / marketPrice) * 100).toFixed(2) : '—'}%
          </Paragraph>
        </div>
      </div>
    </div>
  );
}
