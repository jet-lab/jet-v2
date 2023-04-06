import { useRecoilState, useRecoilValue } from 'recoil';
import { Tabs } from 'antd';
import { useMemo } from 'react';
import { MainConfig } from '@state/config/marginConfig';
import { CurrentOrderTab, CurrentOrderTabAtom, FixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import { OfferLoan } from './offer-loan';
import { LendNow } from './lend-now';
import { WalletTokens } from '@state/user/walletTokens';
import { Accounts } from '@state/user/accounts';
import { UserGuide } from '../shared/user-guide'
import { CopyableField } from '@components/misc/CopyableField';

export const FixedLendOrderEntry = () => {
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
            label: 'offer loan',
            key: 'offer-loan'
          },
          {
            label: 'lend now',
            key: 'lend-now'
          }
        ]}></Tabs>
      {currentTab === 'offer-loan' && (
        <OfferLoan decimals={decimals} token={token} marketAndConfig={marketAndConfig} marginConfig={marginConfig} />
      )}
      {currentTab === 'lend-now' && (
        <LendNow decimals={decimals} token={token} marketAndConfig={marketAndConfig} marginConfig={marginConfig} />
      )}
    </div>
  );
};
