import { useRecoilState, useRecoilValue } from 'recoil';
import { Tabs } from 'antd';
import { useMemo } from 'react';
import { CurrentOrderTab, CurrentOrderTabAtom, FixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import { MainConfig } from '@state/config/marginConfig';
import { RequestLoan } from './request-loan';
import { BorrowNow } from './borrow-now';
import { WalletTokens } from '@state/user/walletTokens';
import { Accounts } from '@state/user/accounts';
import { UserGuide } from '../shared/user-guide'
import { CopyableField } from '@components/misc/CopyableField';

export const FixedBorrowOrderEntry = () => {
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

  if (!decimals || noAccount || !marketAndConfig || !token || !marginConfig) return null;

  return (
    <div className="order-entry fixed-lend-entry view-element view-element-hidden flex column">
      <UserGuide />
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <div className="order-entry-head-top flex-centered">
          <CopyableField content={marketAndConfig.market.address.toBase58()} />
        </div>
      </div>
      <Tabs
        defaultActiveKey="limit"
        activeKey={currentTab}
        onChange={(type: string) => setCurrentTab(type as CurrentOrderTab)}
        items={[
          {
            label: 'borrow request',
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
