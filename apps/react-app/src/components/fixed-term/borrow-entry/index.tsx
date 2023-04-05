import { useRecoilState, useRecoilValue } from 'recoil';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Typography, Tabs } from 'antd';
import { useMemo } from 'react';
import { FixedLendRowOrder } from '@state/views/fixed-term';
import { CurrentOrderTab, CurrentOrderTabAtom, FixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import { MainConfig } from '@state/config/marginConfig';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { RequestLoan } from './request-loan';
import { BorrowNow } from './borrow-now';
import { WalletTokens } from '@state/user/walletTokens';
import { Accounts } from '@state/user/accounts';
import { UserGuide } from '../shared/user-guide'

export const FixedBorrowOrderEntry = () => {
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const marketAndConfig = useRecoilValue(FixedTermMarketAtom);
  const marginConfig = useRecoilValue(MainConfig);
  const walletTokens = useRecoilValue(WalletTokens);
  const accounts = useRecoilValue(Accounts);
  const noAccount = useMemo(() => !walletTokens || !accounts.length, [accounts, walletTokens]);
  const [currentTab, setCurrentTab] = useRecoilState(CurrentOrderTabAtom);

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

  if (!decimals || noAccount || !marketAndConfig || !token || !marginConfig) return null;

  return (
    <div className="order-entry fixed-lend-entry view-element view-element-hidden flex column">
      <UserGuide />
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <ReorderArrows component="fixedLendEntry" order={rowOrder} setOrder={setRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{marketToString(marketAndConfig.config)}</Paragraph>
        </div>
      </div>
      <Tabs
        defaultActiveKey="limit"
        activeKey={currentTab}
        onChange={(type: string) => setCurrentTab(type as CurrentOrderTab)}
        items={[
          {
            label: 'request loan',
            key: 'request-loan'
          },
          {
            label: 'borrow now',
            key: 'borrow-now'
          }
        ]}></Tabs>
      {currentTab === 'request-loan' && (
        <RequestLoan token={token} decimals={decimals} marketAndConfig={marketAndConfig} marginConfig={marginConfig} />
      )}
      {currentTab === 'borrow-now' && (
        <BorrowNow token={token} decimals={decimals} marketAndConfig={marketAndConfig} marginConfig={marginConfig} />
      )}
    </div>
  );
};
