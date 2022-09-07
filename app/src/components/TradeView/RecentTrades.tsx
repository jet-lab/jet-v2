import { useEffect, useState } from 'react';
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { TradeRowOrder } from '../../state/views/views';
import { CurrentMarket } from '../../state/trade/market';
import { PreferredTimeDisplay } from '../../state/settings/settings';
import { OrderPriceString } from '../../state/trade/order';
import { SerumTrade, RecentTrades as SerumRecentTrades, RecentTradesLoaded } from '../../state/trade/recentTrades';
import { MS_PER_DAY, MS_PER_SECOND, unixToLocalTime, unixToUtcTime } from '../../utils/time';
import { useCurrencyFormatting } from '../../utils/currency';
import { animateDataUpdate } from '../../utils/ui';
import { Skeleton, Table, Typography } from 'antd';
import { ReorderArrows } from '../misc/ReorderArrows';

export function RecentTrades(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter } = useCurrencyFormatting();
  const [tradeRowOrder, setTradeRowOrder] = useRecoilState(TradeRowOrder);
  const preferredTimeDisplay = useRecoilValue(PreferredTimeDisplay);
  const currentMarket = useRecoilValue(CurrentMarket);
  const setOrderPriceString = useSetRecoilState(OrderPriceString);
  const recentTrades = useRecoilValue(SerumRecentTrades);
  const tradesLoaded = useRecoilValue(RecentTradesLoaded);
  const [latestTrades, setLatestTrades] = useState<SerumTrade[]>(recentTrades);
  const [highPrice, setHighPrice] = useState<number | null>(null);
  const [lowPrice, setLowPrice] = useState<number | null>(null);
  const { Paragraph, Text } = Typography;

  // Table data
  const tradesTableColumns = [
    {
      title: `${dictionary.common.price} (${currentMarket?.quoteSymbol ?? '—'})`,
      dataIndex: 'price',
      key: 'price',
      align: 'left' as any,
      render: (price: number, trade: SerumTrade) =>
        tradesLoaded ? (
          price ? (
            <Paragraph className="trade-price" type={trade.side === 'buy' ? 'success' : 'danger'}>
              {currencyFormatter(price, false, currentMarket?.quoteDecimals)}
            </Paragraph>
          ) : (
            <Paragraph className="not-available-text" italic>
              {dictionary.common.notAvailable}
            </Paragraph>
          )
        ) : (
          <Skeleton paragraph={false} active />
        )
    },
    {
      title: `${dictionary.common.size} (${currentMarket?.baseSymbol ?? '—'})`,
      dataIndex: 'size',
      key: 'size',
      align: 'right' as any,
      render: (size: number) =>
        tradesLoaded ? (
          size ? (
            currencyFormatter(size, false, currentMarket?.baseDecimals)
          ) : (
            <Paragraph className="not-available-text" italic>
              {dictionary.common.notAvailable}
            </Paragraph>
          )
        ) : (
          <Skeleton className="align-right" paragraph={false} active />
        )
    },
    {
      title: dictionary.common.time,
      dataIndex: 'time',
      key: 'time',
      align: 'right' as any,
      render: (time: number) =>
        tradesLoaded ? (
          time ? (
            preferredTimeDisplay === 'local' ? (
              unixToLocalTime(time / MS_PER_SECOND)
            ) : (
              unixToUtcTime(time / MS_PER_SECOND)
            )
          ) : (
            <Paragraph className="not-available-text" italic>
              {dictionary.common.notAvailable}
            </Paragraph>
          )
        ) : (
          <Skeleton className="align-right" paragraph={false} active />
        )
    }
  ];

  // On new trades
  useEffect(() => {
    if (!recentTrades[0].time) {
      setHighPrice(null);
      setLowPrice(null);
    }

    // Get new 24 hour high & low
    const pastDayTrades = recentTrades.filter(trade => trade.time > Date.now() - MS_PER_DAY);
    if (pastDayTrades.length) {
      const newHighPrice = pastDayTrades.reduce((prev, current) => (prev.price > current.price ? prev : current)).price;
      const newLowPrice = pastDayTrades.reduce((prev, current) => (prev.price < current.price ? prev : current)).price;
      if (highPrice !== newHighPrice) {
        setHighPrice(newHighPrice);
        animateDataUpdate('flash-opacity', '.high-price');
      }
      if (lowPrice !== newLowPrice) {
        setLowPrice(newLowPrice);
        animateDataUpdate('flash-opacity', '.low-price');
      }
    }

    // If new trade(s)
    if (latestTrades[0].time !== recentTrades[0].time) {
      setLatestTrades(recentTrades);
      // This prevents us from animating on market pair switch
      if (recentTrades[0].time) {
        setTimeout(() => {
          animateDataUpdate(
            `flash-background-${recentTrades[0].side === 'buy' ? 'success' : 'danger'}`,
            '.recent-trades tr:first-of-type td'
          );
        }, 50);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [recentTrades]);

  return (
    <div className="recent-trades view-element view-element-hidden flex column">
      <div className="recent-trades-head view-element-item view-element-item-hidden">
        <ReorderArrows component="recentTrades" order={tradeRowOrder} setOrder={setTradeRowOrder} />
        <div className="recent-trades-head-top flex-centered">
          <Paragraph className="recent-trades-head-top-title">{dictionary.tradeView.recentTrades.title}</Paragraph>
        </div>
        <div className="recent-trades-head-bottom flex-centered">
          <Text className="high-price">{highPrice ? currencyFormatter(highPrice, false, 3) : '—'}</Text>
          <Text italic type="secondary">
            {dictionary.tradeView.recentTrades.twoFourH}
          </Text>
          <Text className="low-price">{lowPrice ? currencyFormatter(lowPrice, false, 3) : '—'}</Text>
        </div>
      </div>
      <div className="recent-trades-body view-element-item view-element-item-hidden">
        <Table
          dataSource={latestTrades}
          columns={tradesTableColumns}
          className="no-row-interaction"
          rowKey={row => `${row.orderId}-${Math.random()}`}
          rowClassName={row => (!row.time || !tradesLoaded ? 'no-interaction' : '')}
          onRow={(row: SerumTrade) => {
            return {
              onClick: () => setOrderPriceString(row.price.toString())
            };
          }}
          pagination={false}
        />
      </div>
    </div>
  );
}
