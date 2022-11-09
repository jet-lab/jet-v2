import { useRecoilState, useRecoilValue } from 'recoil';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Tabs, Typography } from 'antd';
import { useMemo, useState } from 'react';
import { MainConfig } from '@state/config/marginConfig';
import { FixedLendRowOrder } from '@state/views/fixed-term';
import { FixedMarketAtom } from '@state/fixed-market/fixed-term-market-sync';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { OfferLoan } from './offer-loan';
import { LendNow } from './lend-now';
import { WalletTokens } from '@state/user/walletTokens';
import { Accounts } from '@state/user/accounts';

export const FixedLendOrderEntry = () => {
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const marketAndConfig = useRecoilValue(FixedMarketAtom);
  const marginConfig = useRecoilValue(MainConfig);
  const walletTokens = useRecoilValue(WalletTokens);
  const accounts = useRecoilValue(Accounts);
  const noAccount = useMemo(() => !walletTokens || !accounts.length, [accounts, walletTokens]);

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
  if (!decimals || noAccount) return null;

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
            label: 'offer loan',
            key: 'limit'
          },
          {
            label: 'lend now',
            key: 'market'
          }
        ]}></Tabs>
      {orderType === 'limit' && (
        <OfferLoan decimals={decimals} token={token} marketAndConfig={marketAndConfig} marginConfig={marginConfig} />
      )}
      {orderType === 'market' && (
        <LendNow decimals={decimals} token={token} marketAndConfig={marketAndConfig} marginConfig={marginConfig} />
      )}
    </div>
  );
};
