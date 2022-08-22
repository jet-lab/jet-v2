import { useLocation } from 'react-router-dom';
import { useRecoilState, useSetRecoilState, useRecoilValue, SetterOrUpdater } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { PoolAction } from '@jet-lab/margin';
import { Dictionary } from '../../state/settings/localization/localization';
import { TradeViewOrder, PoolsViewOrder, AccountsViewOrder } from '../../state/views/views';
import { WalletInit } from '../../state/user/walletTokens';
import { WalletModal, EditAccountModal, NewAccountModal } from '../../state/modals/modals';
import {
  Accounts,
  AccountsInit,
  CurrentAccountName,
  FavoriteAccounts,
  useAccountNames,
  CurrentAccount
} from '../../state/user/accounts';
import { actionOptions, CurrentAction } from '../../state/actions/actions';
import { useCurrencyFormatting } from '../../utils/currency';
import { formatRiskIndicator } from '../../utils/format';
import { useRiskLevel } from '../../utils/risk';
import { Typography, Button, Dropdown, Menu, Tabs, Skeleton } from 'antd';
import { EditOutlined, StarFilled, StarOutlined } from '@ant-design/icons';
import { ReorderArrows } from './ReorderArrows';
import { ConnectionFeedback } from './ConnectionFeedback';
import { Info } from './Info';
import { RiskMeter } from './RiskMeter';
import { ActionIcon } from './ActionIcon';
import { ReactComponent as AngleDown } from '../../styles/icons/arrow-angle-down.svg';

export function AccountSnapshot(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const { pathname } = useLocation();
  const [tradeViewOrder, setTradeViewOrder] = useRecoilState(TradeViewOrder);
  const [poolsViewOrder, setPoolsViewOrder] = useRecoilState(PoolsViewOrder);
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const setWalletModalOpen = useSetRecoilState(WalletModal);
  const { connected, publicKey } = useWallet();
  const walletInit = useRecoilValue(WalletInit);
  const accounts = useRecoilValue(Accounts);
  const accountsInit = useRecoilValue(AccountsInit);
  const [currentAccountName, setCurrentAccountName] = useRecoilState(CurrentAccountName);
  const currentAccount = useRecoilValue(CurrentAccount);
  const requiredCollateral = currentAccount?.valuation.requiredCollateral.asNumber() ?? 0;
  const effectiveCollateral = currentAccount?.valuation.effectiveCollateral.asNumber() ?? 0;
  const accountNames = useAccountNames();
  const [favoriteAccounts, setFavoriteAccounts] = useRecoilState(FavoriteAccounts);
  const walletFavoriteAccounts = publicKey ? favoriteAccounts[publicKey.toString()] ?? [] : [];
  const [newAccountModalOpen, setNewAccountModalOpen] = useRecoilState(NewAccountModal);
  const setEditAccountModalOpen = useSetRecoilState(EditAccountModal);
  const [currentAction, setCurrentAction] = useRecoilState(CurrentAction);
  const riskLevel = useRiskLevel();
  const { Title, Text } = Typography;
  const { TabPane } = Tabs;

  // Determing which reordering state to utilize
  function getOrderContext(): {
    order: string[];
    setOrder: SetterOrUpdater<string[]>;
  } {
    if (pathname === '/') {
      return {
        order: tradeViewOrder,
        setOrder: setTradeViewOrder
      };
    } else if (pathname === '/borrow') {
      return {
        order: poolsViewOrder,
        setOrder: setPoolsViewOrder
      };
    } else {
      return {
        order: accountsViewOrder,
        setOrder: setAccountsViewOrder
      };
    }
  }

  // Either set current action or prompt wallet connection (if not connected)
  function setActionOrConnect(action?: PoolAction) {
    if (connected && walletInit && !accounts.length) {
      setNewAccountModalOpen(true);
    } else if (connected && walletInit) {
      setCurrentAction(action);
    } else {
      setWalletModalOpen(true);
    }
  }

  // Update favorite accounts
  function updateFavoriteAccounts(accountName: string, remove?: boolean) {
    if (!publicKey) {
      return;
    }

    const favoriteAccountsClone = { ...favoriteAccounts };
    const favoriteWalletAccounts = favoriteAccountsClone[publicKey.toString()] ?? [];
    const newFavorites: string[] = [...favoriteWalletAccounts];
    if (remove) {
      const accountIndex = newFavorites.indexOf(accountName);
      if (accountIndex > -1) {
        newFavorites.splice(accountIndex, 1);
      }
    } else {
      newFavorites.push(accountName);
    }
    favoriteAccountsClone[publicKey.toString()] = newFavorites;
    setFavoriteAccounts(favoriteAccountsClone);
  }

  return (
    <div className="account-snapshot view-element view-element-hidden flex-centered column">
      <div className="account-snapshot-head view-element-item view-element-item-hidden flex align-center justify-between">
        <div className="account-snapshot-head-tabs flex align-center justify-start">
          <StarFilled />
          {walletInit && accountNames.length ? (
            <Tabs
              activeKey={currentAccountName}
              onChange={(name: string) => setCurrentAccountName(name)}
              className={
                !currentAccountName || !walletFavoriteAccounts.includes(currentAccountName) ? 'no-active-tabs' : ''
              }>
              {walletFavoriteAccounts.map(name => (
                <TabPane
                  key={name}
                  tab={name.includes('...') ? name : name.toUpperCase()}
                  active={currentAccountName?.toLocaleLowerCase() === name.toLocaleUpperCase()}></TabPane>
              ))}
            </Tabs>
          ) : (
            <></>
          )}
        </div>
        <div className="account-snapshot-head-accounts flex-centered">
          <Button
            className={`function-btn ${newAccountModalOpen ? 'active' : ''}`}
            disabled={!walletInit || !accountsInit}
            onClick={() => setNewAccountModalOpen(true)}>
            {dictionary.actions.newAccount.title} +
          </Button>
          <Dropdown
            disabled={!walletInit || !accountNames.length}
            trigger={['click']}
            overlay={
              <Menu className="all-accounts-menu">
                {accountNames.map(name => (
                  <Menu.Item
                    key={name}
                    onClick={() => setCurrentAccountName(name)}
                    className={name === currentAccountName ? 'active' : ''}>
                    {connected && (
                      <>
                        <div className="all-accounts-menu-name flex align-center justify-start">
                          {walletFavoriteAccounts.includes(name) ? (
                            <StarFilled style={{ opacity: 1 }} onClick={() => updateFavoriteAccounts(name, true)} />
                          ) : (
                            <StarOutlined onClick={() => updateFavoriteAccounts(name)} />
                          )}
                          {name}
                        </div>
                        <EditOutlined
                          onClick={() => {
                            setCurrentAccountName(name);
                            setEditAccountModalOpen(true);
                          }}
                        />
                      </>
                    )}
                  </Menu.Item>
                ))}
              </Menu>
            }>
            <Text type="secondary">
              {dictionary.accountSnapshot.allAccounts.toUpperCase()}
              <AngleDown className="jet-icon" />
            </Text>
          </Dropdown>
        </div>
      </div>
      <div className="account-snapshot-body view-element-item view-element-item-hidden flex justify-center align-start wrap">
        <div className="account-snapshot-body-item flex-centered column">
          <Info term="accountValue">
            <Text className="small-accent-text info-element">{dictionary.common.accountBalance}</Text>
          </Info>
          {walletInit && !accountsInit ? (
            <Skeleton className="align-center" paragraph={false} active />
          ) : (
            <Title>
              {currencyFormatter(
                currentAccount?.summary
                  ? currentAccount.summary.depositedValue - currentAccount.summary.borrowedValue
                  : 0,
                true
              )}
            </Title>
          )}
          <div className="assets-liabilities flex-centered">
            <Text type="success">
              {dictionary.common.assets}
              {walletInit &&
                accountsInit &&
                `: ${currencyAbbrev(
                  currentAccount?.summary.depositedValue ? currentAccount.summary.depositedValue : 0,
                  true
                )}`}
            </Text>
            <div className="assets-liabilities-divider"></div>
            <Text type="danger">
              {dictionary.accountSnapshot.liabilities}
              {walletInit &&
                accountsInit &&
                `: ${currencyAbbrev(
                  currentAccount?.summary.borrowedValue ? currentAccount.summary.borrowedValue : 0,
                  true
                )}`}
            </Text>
          </div>
        </div>
        <div className="account-snapshot-body-item flex-centered column">
          <Info term="requiredCollateral">
            <Text className="small-accent-text info-element">{dictionary.common.requiredCollateral}</Text>
          </Info>
          {walletInit && !accountsInit ? (
            <Skeleton className="align-center" paragraph={false} active />
          ) : (
            <Title>{currencyAbbrev(requiredCollateral, true)}</Title>
          )}
        </div>
        <div className="account-snapshot-body-item flex-centered column">
          <Info term="effectiveCollateral">
            <Text className="small-accent-text info-element">{dictionary.common.effectiveCollateral}</Text>
          </Info>
          {walletInit && !accountsInit ? (
            <Skeleton className="align-center" paragraph={false} active />
          ) : (
            <Title>{currencyAbbrev(effectiveCollateral, true)}</Title>
          )}
        </div>
        <div className="account-snapshot-body-item flex-centered column">
          <Info term="riskLevel">
            <Text className="small-accent-text info-element">{dictionary.common.riskLevel}</Text>
          </Info>
          {walletInit && !accountsInit ? (
            <Skeleton className="align-center" paragraph={false} active />
          ) : (
            <Title className={`risk-level-${riskLevel}`}>
              {formatRiskIndicator(currentAccount?.riskIndicator ?? 0)}
            </Title>
          )}
          <Text type="secondary" italic>
            {walletInit && accountsInit ? dictionary.accountsView.riskMeter[`${riskLevel}Detail`] : ''}
          </Text>
          <RiskMeter />
        </div>
        <ConnectionFeedback />
      </div>
      <div className="account-snapshot-footer view-element-item view-element-item-hidden flex-centered">
        {actionOptions.map(action => (
          <Button
            key={action}
            className={currentAction === action ? 'active' : ''}
            onClick={() => setActionOrConnect(action)}
            disabled={!walletInit || !accountsInit}>
            <ActionIcon action={action} />
            {dictionary.actions[action].title}
          </Button>
        ))}
      </div>
      <ReorderArrows
        component="accountSnapshot"
        order={getOrderContext().order}
        setOrder={getOrderContext().setOrder}
        vertical
      />
    </div>
  );
}
