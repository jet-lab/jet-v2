import { useRecoilState, useRecoilValue } from 'recoil';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Typography, Tabs } from 'antd';
import { useMemo, useState } from 'react';
import { FixedLendRowOrder } from '@state/views/fixed-term';
import { FixedMarketAtom } from '@state/fixed/fixed-term-market-sync';
import { MainConfig } from '@state/config/marginConfig';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { RequestLoan } from './request-loan';
import { BorrowNow } from './borrow-now';

export const FixedBorrowOrderEntry = () => {
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const marketAndConfig = useRecoilValue(FixedMarketAtom);
  const marginConfig = useRecoilValue(MainConfig);
  const [orderType, setOrderType] = useState('limit');

  const token = useMemo(() => {
    if (!marginConfig || !marketAndConfig) return null;
    return Object.values(marginConfig?.tokens).find(token => {
      return marketAndConfig.config.underlyingTokenMint === token.mint.toString();
    });
  }, [marginConfig, marketAndConfig?.config]);

  const decimals = useMemo(() => {
    if (!token) return null;
    if (!marginConfig || !marketAndConfig?.config) return 6;
    return token.decimals;
  }, [token]);

  const { Paragraph } = Typography;

  if (!decimals) return null;

  return (
    <div className="order-entry fixed-lend-entry view-element view-element-hidden flex column">
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <ReorderArrows component="fixedLendEntry" order={rowOrder} setOrder={setRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{marketToString(marketAndConfig.config)}</Paragraph>
        </div>
      </div>
      <Tabs
        defaultActiveKey="limit"
        activeKey={orderType}
        onChange={type => setOrderType(type)}
        items={[
          {
            label: 'request loan',
            key: 'limit'
          },
          {
            label: 'borrow now',
            key: 'market'
          }
        ]}></Tabs>
      {orderType === 'limit' && <RequestLoan token={token} decimals={decimals} marketAndConfig={marketAndConfig} marginConfig={marginConfig} />}
      {orderType === 'market' && <BorrowNow token={token} decimals={decimals} marketAndConfig={marketAndConfig} marginConfig={marginConfig} />}
    </div>
  );
};
